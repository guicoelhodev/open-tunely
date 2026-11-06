import { test, afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'svelte/store';
import { attachPlayer, playTrack, player } from '../src/lib/services/player.service.ts';
let detach;
afterEach(() => { detach?.(); detach = undefined; });
function audioElement(play) {
  const audio = { src: '', error: null, pause() {}, removeAttribute() { this.src = ''; }, load() { this.error = null; }, play() { return play(this); } };
  detach = attachPlayer(audio);
  return audio;
}
function preview(load) { return { id: 'preview', title: 'Preview', refreshOnPlaybackError: true, load }; }
test('refreshes a temporary URL once when media playback fails', async () => {
  let loads = 0, plays = 0, disposed = 0;
  const audio = audioElement(async a => { if (++plays === 1) { a.error = { code: 4 }; throw new Error('Rejected stream'); } });
  await playTrack(preview(async () => ({ url: `https://audio.invalid/${++loads}`, dispose() { disposed++; } })));
  assert.equal(loads, 2); assert.equal(plays, 2); assert.equal(disposed, 1);
  assert.equal(audio.src, 'https://audio.invalid/2'); assert.equal(get(player).error, '');
});
test('stops retrying after a second media failure', async () => {
  let loads = 0;
  audioElement(async a => { a.error = { code: 2 }; throw new Error('Network failure'); });
  await playTrack(preview(async () => ({ url: `https://audio.invalid/${++loads}`, dispose() {} })));
  assert.equal(loads, 2); assert.match(get(player).error, /Network failure/);
});
test('does not retry an autoplay rejection', async () => {
  let loads = 0;
  audioElement(async () => { throw new DOMException('Permission denied', 'NotAllowedError'); });
  await playTrack(preview(async () => { loads++; return { url: 'https://audio.invalid/', dispose() {} }; }));
  assert.equal(loads, 1); assert.match(get(player).error, /NotAllowedError/);
});
test('does not retry source extraction failures', async () => {
  let loads = 0;
  audioElement(async () => {});
  await playTrack(preview(async () => { loads++; throw new Error('Extraction failed'); }));
  assert.equal(loads, 1); assert.match(get(player).error, /Extraction failed/);
});
test('does not retry local tracks', async () => {
  let loads = 0;
  audioElement(async a => { a.error = { code: 3 }; throw new Error('Invalid MP3'); });
  await playTrack({ id: 'local', title: 'Local', load: async () => { loads++; return { url: 'http://127.0.0.1/audio', dispose() {} }; } });
  assert.equal(loads, 1);
});
test('disposes a stale refreshed URL without replacing a newer track', async () => {
  let loads = 0, finish, disposed = 0;
  audioElement(async a => { if (a.src.includes('preview')) { a.error = { code: 2 }; throw new Error('Network failure'); } });
  const pending = playTrack(preview(async () => {
    if (++loads === 1) return { url: 'https://audio.invalid/preview', dispose() {} };
    return await new Promise(resolve => { finish = resolve; });
  }));
  while (!finish) await new Promise(resolve => setImmediate(resolve));
  await playTrack({ id: 'new', title: 'New', load: async () => ({ url: 'http://127.0.0.1/new', dispose() {} }) });
  finish({ url: 'https://audio.invalid/stale', dispose() { disposed++; } });
  await pending;
  assert.equal(disposed, 1); assert.equal(get(player).track.id, 'new'); assert.equal(get(player).url, 'http://127.0.0.1/new');
});
