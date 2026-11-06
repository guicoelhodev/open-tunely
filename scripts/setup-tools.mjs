import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import { chmod, mkdir, mkdtemp, readFile, rename, rm, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const directory = new URL("../src-tauri/binaries/", import.meta.url);
const digest = (data) => createHash("sha256").update(data).digest("hex");
const execute = promisify(execFile);
const denoVersion = "2.9.8";
const windows = process.platform === "win32";
const suffix = windows ? ".exe" : "";
const denoArchiveHash = windows ? "cba84b26e0df5ae7f74fc66b86a95b866872a86292ebd84fa1e959927edd93bf" : "e890975bc2549807bbcc0b584ffafda72cc7c93349dd99c80483dd784f6c4609";
const denoBinaryHash = windows ? "924397800533d3eb0ed0c889b20ad70e18b970d93acd8135bc043f425e528266" : "368bca08999e02c6911b8f248167b582276b1f547db5712b6e736d7eda1dbda7";
const denoLicenseHash = "f62497fffecc0852960c8d3e6934b9db86d16396e9b604072e923892cae3a588";
const ytVersion = "2026.10.26";
const ytHash = windows ? "f5389d3698ecde4ef06f4e6b1ce0294fe9ac1396228c8990b8d3416d6d177a7d" : "767ba9dce7052aa196f8b12652f433d76de128ba168966fa84b6394d5e6c6795";

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
  const binary = new URL(`yt-dlp${suffix}`, directory);
  if (!await verified(binary, ytHash)) {
    console.log(`Baixando yt-dlp ${ytVersion} para ${windows ? "Windows" : "Linux"} x64…`);
    const data = await download(`https://github.com/yt-dlp/yt-dlp/releases/download/${ytVersion}/${windows ? "yt-dlp.exe" : "yt-dlp_linux"}`, ytHash);
    const temporary = `${work}/yt-dlp`;
    await writeFile(temporary, data, { mode: 0o755 });
    await chmod(temporary, 0o755);
    await rename(temporary, binary);
  }
  await chmod(binary, 0o755);
  console.log(`yt-dlp ${ytVersion} preparado.`);
}

async function setupDeno(work) {
  const binary = new URL(`deno${suffix}`, directory);
  const license = new URL("deno-LICENSE.md", directory);
  if (!await verified(binary, denoBinaryHash)) {
    console.log(`Baixando Deno ${denoVersion} para ${windows ? "Windows" : "Linux"} x64…`);
    const data = await download(`https://github.com/denoland/deno/releases/download/v${denoVersion}/deno-x86_64-${windows ? "pc-windows-msvc" : "unknown-linux-gnu"}.zip`, denoArchiveHash);
    const archive = `${work}/deno.zip`;
    await writeFile(archive, data);
    if (windows) {
      await execute("tar", ["-xf", archive, "-C", work]);
    } else {
      await execute("unzip", ["-q", archive, "deno", "-d", work]);
    }
    const extracted = `${work}/deno${suffix}`;
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

async function setupFfmpeg() {
  if (windows) {
    // Build under MSYS2 first; do not accidentally select Git Bash or a full FFmpeg build.
    for (const tool of ["ffmpeg.exe", "ffprobe.exe", "ffmpeg-LICENSE.txt"]) {
      await readFile(new URL(tool, directory));
    }
    await execute(process.execPath, [fileURLToPath(new URL("./check-media-tools.mjs", import.meta.url))]);
    return;
  }
  // Build only the audio features needed by this application; never fetch full builds.
  await execute("bash", [fileURLToPath(new URL("./build-ffmpeg.sh", import.meta.url))], {
    maxBuffer: 16 * 1024 * 1024,
  });
  await execute(process.execPath, [fileURLToPath(new URL("./check-media-tools.mjs", import.meta.url))]);
  console.log("FFmpeg/ffprobe enxutos preparados e conversão validada.");
}

if (!["linux", "win32"].includes(process.platform) || process.arch !== "x64") {
  throw new Error("Por enquanto, as ferramentas estão disponíveis para Linux x64 e Windows x64.");
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
