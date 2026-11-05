import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, rename, rm, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const directory = new URL("../src-tauri/binaries/", import.meta.url);
const digest = (data) => createHash("sha256").update(data).digest("hex");
const execute = promisify(execFile);
const denoVersion = "2.9.8";
const denoArchiveHash = "e890975bc2549807bbcc0b584ffafda72cc7c93349dd99c80483dd784f6c4609";
const denoBinaryHash = "368bca08999e02c6911b8f248167b582276b1f547db5712b6e736d7eda1dbda7";
const denoLicenseHash = "f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588";
const ytVersion = "2026.10.26";
const ytHash = "767ba9dce7052aa196f8b12652f433d76de128ba168966fa84b6394d5e6c6795";
// Static Linux x64 build maintained by yt-dlp; pinned tag and archive SHA-256.
const ffVersion = "autobuild-2026-11-03-20-37";
const ffRoot = "ffmpeg-N-128383-g47f35c191c-linux64-gpl";
const ffArchiveHash = "51cc2787958e5b2fdebdf45be629f69dcd06bf8ff156bda77d8759d95e90e493";
const ffHashes = {
  ffmpeg: "e2430c93fd8c781439b95a238665ab06205f1ce464c7681e7e9c3990331fb162",
  ffprobe: "5de72581e365f5878f54bd257ba5728ef9da3da1a8d6522ed01af93447a8e35f",
};

async function verified(path, hash) {
  try { return digest(await readFile(path)) === hash; }
  catch (error) { if (error.code === "ENOENT") return false; throw error; }
}

async function download(url, hash) {
  const response = await fetch(url, { signal: AbortSignal.timeout(180_000) });
  if (!response.ok) throw new Error(`Download falhou: HTTP ${response.status}`);
  const data = Buffer.from(await response.arrayBuffer());
  if (digest(data) !== hash) throw new Error("Checksum da ferramenta não confere.");
  return data;
}

async function setupYtdlp(work) {
  const binary = new URL("yt-dlp", directory);
  if (!await verified(binary, ytHash)) {
    console.log(`Baixando yt-dlp ${ytVersion} para Linux x64…`);
    const data = await download(`https://github.com/yt-dlp/yt-dlp/releases/download/${ytVersion}/yt-dlp_linux`, ytHash);
    const temporary = `${work}/yt-dlp`;
    await writeFile(temporary, data, { mode: 0o755 });
    await chmod(temporary, 0o755);
    await rename(temporary, binary);
  }
  await chmod(binary, 0o755);
  console.log(`yt-dlp ${ytVersion} preparado.`);
}

async function setupDeno(work) {
  const binary = new URL("deno", directory);
  const license = new URL("deno-LICENSE.md", directory);
  if (!await verified(binary, denoBinaryHash)) {
    console.log(`Baixando Deno ${denoVersion} para Linux x64…`);
    const data = await download(`https://github.com/denoland/deno/releases/download/v${denoVersion}/deno-x86_64-unknown-linux-gnu.zip`, denoArchiveHash);
    const archive = `${work}/deno.zip`;
    await writeFile(archive, data);
    await execute("unzip", ["-q", archive, "deno", "-d", work]);
    const extracted = `${work}/deno`;
    if (!await verified(extracted, denoBinaryHash)) throw new Error("Checksum de deno não confere.");
    await chmod(extracted, 0o755);
    await rename(extracted, binary);
  }
  if (!await verified(license, denoLicenseHash)) {
    const data = await download(`https://raw.githubusercontent.com/denoland/deno/v${denoVersion}/LICENSE.md`, denoLicenseHash);
    const temporary = `${work}/deno-LICENSE.md`;
    await writeFile(temporary, data);
    await rename(temporary, license);
  }
  await chmod(binary, 0o755);
  console.log(`Deno ${denoVersion} preparado.`);
}

async function setupFfmpeg(work) {
  const valid = await Promise.all(Object.entries(ffHashes).map(([name, hash]) => verified(new URL(name, directory), hash)));
  let hasLicense = false;
  try { hasLicense = (await readFile(new URL("ffmpeg-LICENSE.txt", directory))).length > 0; }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (!valid.every(Boolean) || !hasLicense) {
    console.log(`Baixando FFmpeg/ffprobe ${ffVersion} para Linux x64…`);
    const data = await download(`https://github.com/yt-dlp/FFmpeg-Builds/releases/download/${ffVersion}/${ffRoot}.tar.xz`, ffArchiveHash);
    const archive = `${work}/ffmpeg.tar.xz`;
    await writeFile(archive, data);
    // tar/xz is needed only during development setup, never by the desktop app.
    await execute("tar", ["-xJf", archive, "-C", work,
      `${ffRoot}/bin/ffmpeg`, `${ffRoot}/bin/ffprobe`, `${ffRoot}/LICENSE.txt`]);
    for (const [name, hash] of Object.entries(ffHashes)) {
      const extracted = `${work}/${ffRoot}/bin/${name}`;
      if (!await verified(extracted, hash)) throw new Error(`Checksum de ${name} não confere.`);
      await chmod(extracted, 0o755);
      await rename(extracted, new URL(name, directory));
    }
    await rename(`${work}/${ffRoot}/LICENSE.txt`, new URL("ffmpeg-LICENSE.txt", directory));
  }
  for (const name of Object.keys(ffHashes)) await chmod(new URL(name, directory), 0o755);
  console.log(`FFmpeg/ffprobe ${ffVersion} preparados.`);
}

if (process.platform !== "linux" || process.arch !== "x64") {
  throw new Error("Por enquanto, as ferramentas estão disponíveis somente para Linux x64.");
}
let work;
try {
  await mkdir(directory, { recursive: true });
  work = await mkdtemp(`${fileURLToPath(directory)}.setup-`);
  await setupYtdlp(work);
  await setupDeno(work);
  await setupFfmpeg(work);
} catch (error) {
  console.error(`Não foi possível preparar as ferramentas: ${error.message}`);
  process.exitCode = 1;
} finally {
  if (work) await rm(work, { recursive: true, force: true });
}
