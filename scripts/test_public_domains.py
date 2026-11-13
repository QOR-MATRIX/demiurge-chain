# SPDX-License-Identifier: Apache-2.0
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('public_domains', Path(__file__).with_name('check-public-domains.py'))
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)


class PublicDomains(unittest.TestCase):
    def test_tree(self):
        count, found = policy.inspect()
        self.assertGreater(count, 0)
        self.assertEqual(found, [])

    def test_encoded_and_plain_links(self):
        for suffix in ('cloud', 'guru', 'io', 'network'):
            host = 'demiurge' + '.' + suffix
            for value in (host, 'https://RPC.' + host.upper(), host.replace('.', '&#46;'),
                          host.replace('.', '%252E'), host.replace('.', '\uff0e'), 'person@' + host):
                with self.subTest(suffix=suffix):
                    self.assertEqual(policy.locations('safe\n'+value), [2])

    def test_identity_and_source_references(self):
        self.assertEqual(policy.locations('\n'.join([
            'cloud.demiurge.qor', 'cloud.demiurge.qor-launcher', 'demiurge:qor-id:challenge:v1:',
            'https://qorsync.dev', 'wss://rpc.qorsync.dev', 'http://localhost:9944',
            'https://github.com/QOR-MATRIX/demiurge-chain', 'https://github.com/QOR-MATRIX/QOR-OS',
        ])), [])

    def test_long_prefix(self):
        prefix = 'subdomain.' * 10000
        self.assertEqual(policy.locations(prefix+'qorsync.dev'), [])
        self.assertEqual(policy.locations(prefix+'demiurge'+'.cloud'), [1])

    def test_file_size_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'README.md').write_bytes(b'x'*(policy.MAX_TEXT+1))
            with patch.object(policy.subprocess, 'check_output', return_value=b'README.md\0'):
                with self.assertRaises(ValueError):
                    policy.inspect(root)

    def test_restricted_paths_are_not_opened(self):
        with patch.object(policy.subprocess, 'check_output', return_value=b'.env.example\0private-secret.md\0'), \
                patch.object(Path, 'open') as opened:
            self.assertEqual(policy.inspect(), (0, []))
        opened.assert_not_called()

    def test_linked_input_is_refused(self):
        with patch.object(policy.subprocess, 'check_output', return_value=b'README.md\0'), \
                patch.object(Path, 'is_symlink', return_value=True), patch.object(Path, 'open') as opened:
            with self.assertRaises(ValueError):
                policy.inspect()
        opened.assert_not_called()

    def test_invalid_encoding_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'README.md').write_bytes(b'\xff')
            with patch.object(policy.subprocess, 'check_output', return_value=b'README.md\0'):
                with self.assertRaises(ValueError):
                    policy.inspect(root)

    def test_diagnostics_do_not_echo_failed_values(self):
        output = io.StringIO()
        with contextlib.redirect_stderr(output), patch.object(policy, 'inspect', side_effect=ValueError('PRIVATE_TEST_VALUE')):
            self.assertEqual(policy.main(), 2)
        self.assertNotIn('PRIVATE_TEST_VALUE', output.getvalue())


if __name__ == '__main__':
    unittest.main()
