#include "webvoice.h"

#include <algorithm>

#ifdef Q_OS_WASM
#include <QFile>
#include <emscripten.h>

// The page's Web Audio, one context for every voice: globalThis.qqAudio = { ctx, voices, next, start }. A voice is a
// gain and, where the browser has one, a stereo panner, in front of the page's output; its file is decoded once and
// played as often as asked. The first touch or key resumes the context, which a browser holds until then.
EM_JS(int, qqVoiceNew, (), {
    if (!globalThis.qqAudio) {
        const Context = globalThis.AudioContext || globalThis.webkitAudioContext;
        const ctx = Context ? new Context() : null;
        const resume = () => { if (ctx && ctx.state !== 'running') ctx.resume(); };
        for (const kind of ['pointerdown', 'keydown', 'touchstart'])
            globalThis.addEventListener(kind, resume, { capture: true });
        const audio = { ctx: ctx, voices: new Map(), next: 1 };
        audio.start = (v, loops) => {
            if (v.source) {
                v.source.onended = null;
                try { v.source.stop(); } catch (e) {}
            }
            const src = ctx.createBufferSource();
            src.buffer = v.buffer;
            src.loop = loops < 0;
            const left = loops < 0 ? 0 : Math.max(1, loops) - 1;
            src.onended = () => { if (left > 0) audio.start(v, left); };
            src.connect(v.gain);
            src.start();
            v.source = src;
        };
        globalThis.qqAudio = audio;
    }
    const audio = globalThis.qqAudio;
    if (!audio.ctx)
        return 0;
    const ctx = audio.ctx;
    const voice = { gain: ctx.createGain(), panner: ctx.createStereoPanner ? ctx.createStereoPanner() : null,
                    buffer: null, source: null, pending: null, volume: 1, pan: 0 };
    if (voice.panner) {
        voice.gain.connect(voice.panner);
        voice.panner.connect(ctx.destination);
    } else {
        voice.gain.connect(ctx.destination);
    }
    const id = audio.next++;
    audio.voices.set(id, voice);
    return id;
});

// A file's decoding, kept by its URL for the next voice that plays it (a coin's sound, heard again and again, is read
// and decoded once). The last 32 kept; a game's files do not change while it plays.
EM_JS(int, qqVoiceUse, (int id, const char *key, const unsigned char *data, int size), {
    const audio = globalThis.qqAudio;
    const voice = audio && audio.voices.get(id);
    if (!voice)
        return 1;
    const name = UTF8ToString(key);
    voice.buffer = null;
    voice.loading = name;
    if (!name)
        return 1;  // no file: nothing to play
    audio.decoded = audio.decoded || new Map();
    let decoding = audio.decoded.get(name);
    if (!decoding) {
        if (!data)
            return 0;  // not kept: asked again with the file's bytes
        // A copy out of the module's memory: decodeAudioData takes the buffer it is given, and that memory is shared.
        decoding = audio.ctx.decodeAudioData(HEAPU8.slice(data, data + size).buffer);
        decoding.catch(() => audio.decoded.delete(name));
        audio.decoded.set(name, decoding);
        if (audio.decoded.size > 32)
            audio.decoded.delete(audio.decoded.keys().next().value);
    }
    decoding.then((buffer) => {
        if (voice.loading !== name)
            return;  // given another file since
        voice.buffer = buffer;
        if (voice.pending !== null) {
            const loops = voice.pending;
            voice.pending = null;
            audio.start(voice, loops);
        }
    }, () => {});
    return 1;
});

EM_JS(void, qqVoicePlay, (int id, int loops), {
    const audio = globalThis.qqAudio;
    const voice = audio && audio.voices.get(id);
    if (!voice)
        return;
    if (voice.buffer)
        audio.start(voice, loops);
    else
        voice.pending = loops;
});

EM_JS(void, qqVoiceStop, (int id), {
    const audio = globalThis.qqAudio;
    const voice = audio && audio.voices.get(id);
    if (!voice)
        return;
    voice.pending = null;
    if (voice.source) {
        voice.source.onended = null;
        try { voice.source.stop(); } catch (e) {}
        voice.source = null;
    }
});

EM_JS(void, qqVoiceSet, (int id, double volume, double pan), {
    const audio = globalThis.qqAudio;
    const voice = audio && audio.voices.get(id);
    if (!voice)
        return;
    voice.volume = volume;
    voice.pan = pan;
    voice.gain.gain.value = volume;
    if (voice.panner)
        voice.panner.pan.value = pan;
});

EM_JS(void, qqVoiceFree, (int id), {
    const audio = globalThis.qqAudio;
    const voice = audio && audio.voices.get(id);
    if (!voice)
        return;
    if (voice.source) {
        voice.source.onended = null;
        try { voice.source.stop(); } catch (e) {}
    }
    voice.gain.disconnect();
    if (voice.panner)
        voice.panner.disconnect();
    audio.voices.delete(id);
});
#else
#include <QSoundEffect>
#endif

namespace qq {

WebVoice::WebVoice(QObject *parent) : QObject(parent)
{
#ifdef Q_OS_WASM
    m_id = qqVoiceNew();
#else
    m_effect = new QSoundEffect(this);
#endif
}

WebVoice::~WebVoice()
{
#ifdef Q_OS_WASM
    if (m_id)
        qqVoiceFree(m_id);
#endif
}

void WebVoice::setSource(const QUrl &source)
{
    if (source == m_source)
        return;
    m_source = source;
#ifdef Q_OS_WASM
    // What it was playing stops, as a sound effect's does when given another file (or none: a track let go).
    if (m_id)
        qqVoiceStop(m_id);
    // The file's bytes from wherever the game keeps them: its package on the page's file system, or bundled in qrc:.
    const QString path = source.isLocalFile() ? source.toLocalFile()
                       : source.scheme() == QStringLiteral("qrc") ? QStringLiteral(":") + source.path()
                                                                   : QString();
    const QByteArray key = path.isEmpty() ? QByteArray() : source.toString().toUtf8();
    if (m_id && !qqVoiceUse(m_id, key.constData(), nullptr, 0)) {
        QFile f(path);
        if (f.open(QIODevice::ReadOnly)) {
            const QByteArray bytes = f.readAll();
            qqVoiceUse(m_id, key.constData(), reinterpret_cast<const unsigned char *>(bytes.constData()),
                       int(bytes.size()));
        }
    }
#else
    m_effect->setSource(source);
#endif
    emit sourceChanged();
}

void WebVoice::setVolume(qreal volume)
{
    volume = std::clamp(volume, 0.0, 1.0);
    if (volume == m_volume)
        return;
    m_volume = volume;
    apply();
    emit volumeChanged();
}

void WebVoice::setPan(qreal pan)
{
    pan = std::clamp(pan, -1.0, 1.0);
    if (pan == m_pan)
        return;
    m_pan = pan;
    apply();
    emit panChanged();
}

void WebVoice::setLoops(int loops)
{
    if (loops == m_loops)
        return;
    m_loops = loops;
#ifndef Q_OS_WASM
    m_effect->setLoopCount(loops);
#endif
    emit loopsChanged();
}

void WebVoice::apply()
{
#ifdef Q_OS_WASM
    if (m_id)
        qqVoiceSet(m_id, m_volume, m_pan);
#else
    m_effect->setVolume(float(m_volume));  // a QSoundEffect cannot pan
#endif
}

void WebVoice::play()
{
    apply();
#ifdef Q_OS_WASM
    if (m_id)
        qqVoicePlay(m_id, m_loops);
#else
    m_effect->play();
#endif
}

void WebVoice::stop()
{
#ifdef Q_OS_WASM
    if (m_id)
        qqVoiceStop(m_id);
#else
    m_effect->stop();
#endif
}

}  // namespace qq
