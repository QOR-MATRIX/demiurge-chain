/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*
 * The QOR Launcher's translation worker (ADR-092 decision 3): Mozilla's Bergamot engine, run on this PC, for Social.
 *
 * How it sets the engine up (each file copied into aligned memory, the Marian configuration, translating through
 * English with two models) follows Firefox's translations-engine.worker.js (mozilla-firefox/firefox, branch release,
 * commit 3ee4b3c1937c8b77c0827a56e70de386b1490473), which is MPL-2.0, as this file therefore is. Beside it,
 * bergamot-translator.js is Firefox's loader for the engine, v0.6.0+eea6e5a8, unchanged (SHA-256 faff1ef6…7b47d), with
 * its licence in LICENSE. The engine and the models are handed in as bytes by the launcher, which checked them against
 * the hashes Mozilla publishes; this worker fetches nothing.
 *
 * In:  { type: 'init', engine: ArrayBuffer, pairs: [{ from, to, files: { model: { name, bytes }, lex?, vocab?, srcvocab?, trgvocab? } }] }
 *      { type: 'init', mock: true, pairs: [{ from, to }] }   (the interface's own checks only: no engine, words marked)
 *      { type: 'translate', id, text }
 * Out: { type: 'ready' } | { type: 'error', message }
 *      { type: 'done', id, text } | { type: 'failed', id, message }
 */

/* global loadBergamot */
importScripts('bergamot-translator.js');

const ALIGNMENT = { model: 256, lex: 64, vocab: 64, srcvocab: 64, trgvocab: 64 };

/** The Marian configuration Firefox uses: greedy, no quality model, as text with the whitespace Marian wants. */
function configuration(modelName) {
  const entries = {
    'beam-size': '1',
    normalize: '1.0',
    'word-penalty': '0',
    'max-length-break': '128',
    'mini-batch-words': '1024',
    workspace: '128',
    'max-length-factor': '2.0',
    'skip-cost': 'true',
    'cpu-threads': '0',
    quiet: 'true',
    'quiet-translation': 'true',
    'gemm-precision': modelName.endsWith('intgemm8.bin') ? 'int8shiftAll' : 'int8shiftAlphaAll',
    alignment: 'soft',
  };
  const indent = '            ';
  let text = '\n';
  for (const [key, value] of Object.entries(entries)) text += `${indent}${key}: ${value}\n`;
  return text + indent;
}

function start(wasmBinary) {
  return new Promise((resolve, reject) => {
    const bergamot = loadBergamot({
      // 40 MiB to start, growing as needed, as Firefox found uses the least memory.
      INITIAL_MEMORY: 41_943_040,
      print: () => {},
      printErr: () => {},
      onAbort() {
        reject(new Error('The translation engine could not start.'));
      },
      onRuntimeInitialized: async () => {
        await Promise.resolve();
        resolve(bergamot);
      },
      wasmBinary,
    });
  });
}

function model(bergamot, pair) {
  const aligned = {};
  for (const [kind, file] of Object.entries(pair.files)) {
    const alignment = ALIGNMENT[kind];
    if (!alignment) throw new Error(`An unknown kind of model file: ${kind}`);
    const memory = new bergamot.AlignedMemory(file.bytes.byteLength, alignment);
    memory.getByteArrayView().set(new Uint8Array(file.bytes));
    aligned[kind] = memory;
  }
  const vocabularies = new bergamot.AlignedMemoryList();
  if (aligned.vocab) vocabularies.push_back(aligned.vocab);
  else if (aligned.srcvocab && aligned.trgvocab) {
    vocabularies.push_back(aligned.srcvocab);
    vocabularies.push_back(aligned.trgvocab);
  } else throw new Error('A model came without its vocabulary.');
  return new bergamot.TranslationModel(
    pair.from,
    pair.to,
    configuration(pair.files.model.name),
    aligned.model,
    aligned.lex ?? null,
    vocabularies,
    null,
  );
}

/** One engine for one route: one model, or two through English. */
function engineOf(bergamot, pairs) {
  const models = pairs.map((pair) => model(bergamot, pair));
  const service = new bergamot.BlockingService({ cacheSize: 0 });
  return (text) => {
    const messages = new bergamot.VectorString();
    const options = new bergamot.VectorResponseOptions();
    let responses = null;
    try {
      messages.push_back(text);
      options.push_back({ qualityScores: false, alignment: false, html: false });
      responses =
        models.length === 1
          ? service.translate(models[0], messages, options)
          : service.translateViaPivoting(models[0], models[1], messages, options);
      return responses.get(0).getTranslatedText();
    } finally {
      messages.delete();
      options.delete();
      responses?.delete();
    }
  };
}

/** For the interface's checks: no engine, the words marked so a check can see they went through. */
function mockOf(pairs) {
  const from = pairs[0].from;
  const to = pairs[pairs.length - 1].to;
  return (text) => `[${from}→${to}] ${text}`;
}

let translate = null;

self.addEventListener('message', async ({ data }) => {
  if (data.type === 'init') {
    try {
      translate = data.mock ? mockOf(data.pairs) : engineOf(await start(data.engine), data.pairs);
      self.postMessage({ type: 'ready' });
    } catch (e) {
      self.postMessage({ type: 'error', message: e?.message ?? String(e) });
    }
    return;
  }
  if (data.type === 'translate') {
    if (!translate) {
      self.postMessage({ type: 'failed', id: data.id, message: 'The translation engine is not ready.' });
      return;
    }
    try {
      // Leading and trailing whitespace confuses the models; it is put back around the result.
      const [, before, words, after] = /^(\s*)([\s\S]*?)(\s*)$/.exec(data.text);
      const text = words ? translate(words.replaceAll('­', '')) : '';
      self.postMessage({ type: 'done', id: data.id, text: before + text + after });
    } catch (e) {
      self.postMessage({ type: 'failed', id: data.id, message: e?.message ?? String(e) });
    }
  }
});
