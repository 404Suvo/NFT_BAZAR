import assert from 'node:assert/strict';
import test from 'node:test';

import { parseLumens } from '../lib/constants';
import { cn, getImageUrl, truncateAddress } from '../lib/utils';

test('truncateAddress preserves the account prefix and suffix', () => {
  assert.equal(truncateAddress('GABCDEFGHIJKLMNOP', 4), 'GABCDE...MNOP');
  assert.equal(truncateAddress(''), '');
});

test('getImageUrl resolves IPFS and HTTP metadata URIs', () => {
  assert.equal(
    getImageUrl('ipfs://bafy-test-image'),
    'https://ipfs.io/ipfs/bafy-test-image',
  );
  assert.equal(
    getImageUrl('https://images.example/nft.png'),
    'https://images.example/nft.png',
  );
});

test('getImageUrl supplies a deterministic placeholder for missing metadata', () => {
  assert.equal(getImageUrl(), 'https://picsum.photos/seed/nft/400/400');
});

test('parseLumens converts stroops to a display value', () => {
  assert.equal(parseLumens('12345678'), '1.2345678');
  assert.equal(parseLumens('25000000'), '2.5');
  assert.equal(cn('nft-card', { active: true, hidden: false }), 'nft-card active');
});
