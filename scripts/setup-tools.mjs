import { createHash } from "node:crypto";
import { chmod, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";

// Official standalone Linux x64 build; no system Python or yt-dlp required.
const version = "2026.10.26";
const sha256 = "767ba9dce7052aa196f8b12652f433d76de128ba168966fa84b6394d5e6c6795";
const directory = new URL("../src-tauri/binaries/", import.meta.url);
const binary = new URL("yt-dlp", directory);
const temporary = new URL("yt-dlp.part", directory);
const digest = (data) => createHash("sha256").update(data).digest("hex");

if (process.platform !== "linux" || process.arch !== "x64") {
  throw new Error("Por enquanto, as ferramentas estão disponíveis somente para Linux x64.");
}
try {
  let existing;
  try { existing = await readFile(binary); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (existing && digest(existing) === sha256) {
    await chmod(binary, 0o755);
    console.log(`yt-dlp ${version} já está preparado.`);
  } else {
    console.log(`Baixando yt-dlp ${version} para Linux x64…`);
    const response = await fetch(
      `https://github.com/yt-dlp/yt-dlp/releases/download/${version}/yt-dlp_linux`,
      { signal: AbortSignal.timeout(180_000) },
    );
    if (!response.ok) throw new Error(`Download falhou: HTTP ${response.status}`);
    const data = Buffer.from(await response.arrayBuffer());
    if (digest(data) !== sha256) throw new Error("Checksum do yt-dlp não confere.");
    await mkdir(directory, { recursive: true });
    await writeFile(temporary, data, { mode: 0o755 });
    await chmod(temporary, 0o755);
    await rename(temporary, binary);
    console.log(`yt-dlp ${version} preparado.`);
  }
} catch (error) {
  await rm(temporary, { force: true });
  console.error(`Não foi possível preparar o yt-dlp: ${error.message}`);
  process.exitCode = 1;
}
