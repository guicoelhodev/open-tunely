// Offline integration check for the actual yt-dlp -> FFmpeg postprocessor contract.
import { mkdtemp, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const tools = resolve(process.argv[2] ?? fileURLToPath(new URL('../src-tauri/binaries/', import.meta.url)));
const suffix = process.platform === 'win32' ? '.exe' : '';
const work = await mkdtemp(join(tmpdir(), 'tunely-tools-check-'));
try {
  const samples = 44100;
  const wav = Buffer.alloc(44 + samples * 2);
  wav.write('RIFF'); wav.writeUInt32LE(wav.length - 8, 4); wav.write('WAVEfmt ', 8);
  wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20); wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(44100, 24); wav.writeUInt32LE(88200, 28);
  wav.writeUInt16LE(2, 32); wav.writeUInt16LE(16, 34);
  wav.write('data', 36); wav.writeUInt32LE(samples * 2, 40);
  for (let i = 0; i < samples; i++) wav.writeInt16LE(Math.round(8000 * Math.sin(2 * Math.PI * 440 * i / 44100)), 44 + i * 2);
  const source = join(work, 'fixture.wav');
  await writeFile(source, wav);
  // Existing source file exercises postprocessing without any network requests.
  const info = join(work, 'fixture.info.json');
  await writeFile(info, JSON.stringify({
    id: 'fixture', title: 'fixture', ext: 'wav', format_id: 'wav',
    url: 'https://example.invalid/fixture.wav', extractor: 'generic',
    webpage_url: 'https://example.invalid/fixture', acodec: 'pcm_s16le', vcodec: 'none',
  }));
  execFileSync(join(tools, `yt-dlp${suffix}`), [
    '--ignore-config', '--no-plugin-dirs', '--load-info-json', info,
    '--extract-audio', '--audio-format', 'mp3', '--audio-quality', '0',
    '--ffmpeg-location', tools, '--output', join(work, 'fixture.%(ext)s'),
  ], { stdio: 'inherit', timeout: 60000 });
  const output = join(work, 'fixture.mp3');
  if ((await readFile(output)).length === 0) throw new Error('Empty MP3');
  const result = JSON.parse(execFileSync(join(tools, `ffprobe${suffix}`), [
    '-v', 'error', '-show_streams', '-show_format', '-of', 'json', output,
  ], { encoding: 'utf8', timeout: 10000 }));
  if (result.streams[0]?.codec_name !== 'mp3' || Number(result.format.duration) < 0.9) {
    throw new Error('Invalid MP3 conversion');
  }
  console.log('yt-dlp -> FFmpeg -> MP3 integration passed.');
} finally {
  await rm(work, { recursive: true, force: true });
}
