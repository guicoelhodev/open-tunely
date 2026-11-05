import { m } from "$lib/paraglide/messages.js";
import { getLocale, type Locale } from "$lib/paraglide/runtime.js";

export function errorMessage(cause: unknown, locale: Locale = getLocale()): string {
  const text = String(cause);
  const code = text.split("|")[0];
  switch (code) {
    case "ERR_QUERY_INVALID": return m.error_query_invalid({}, { locale });
    case "ERR_URL_INVALID": return m.error_url_invalid({}, { locale });
    case "ERR_NO_RESULTS": return m.error_no_results({}, { locale });
    case "ERR_SEARCH_CONNECTION": return m.error_search_connection({}, { locale });
    case "ERR_NAME_INVALID": return m.error_name_invalid({}, { locale });
    case "ERR_NAME_EXISTS": return m.error_name_exists({}, { locale });
    case "ERR_DOWNLOAD_EXISTS": return m.error_download_exists({}, { locale });
    case "ERR_DOWNLOAD_CONNECTION": return m.error_download_connection({}, { locale });
    case "ERR_SAVE_PERMISSION": return m.error_save_permission({}, { locale });
    case "ERR_WRITE_PERMISSION": return m.error_write_permission({}, { locale });
    case "ERR_RENAME_PERMISSION": return m.error_rename_permission({}, { locale });
    case "ERR_DELETE_PERMISSION": return m.error_delete_permission({}, { locale });
    case "ERR_READ_PERMISSION": return m.error_read_permission({}, { locale });
    case "ERR_FOLDER_MISSING": return m.error_folder_missing({}, { locale });
    case "ERR_FILE_MISSING": return m.error_file_missing({}, { locale });
    case "ERR_DOWNLOAD_BUSY": return m.error_download_busy({}, { locale });
    default: return text.includes("|") ? text.slice(text.indexOf("|") + 1) : text;
  }
}
