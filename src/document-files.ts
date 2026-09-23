import { execFile } from 'node:child_process'
import { readFile, writeFile } from 'node:fs/promises'
import { basename } from 'node:path'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)

export async function readDocument(path: string) {
  return readFile(path, 'utf8')
}

export async function writeDocument(path: string, text: string) {
  await writeFile(path, text, 'utf8')
}

export function getDocumentName(path: string) {
  return basename(path)
}

export async function chooseSavePath(defaultName: string) {
  const script = `on run argv
try
set selectedFile to choose file name with prompt "Save document as:" default name (item 1 of argv)
return POSIX path of selectedFile
on error number -128
return ""
end try
end run`
  const { stdout } = await execFileAsync('osascript', ['-e', script, defaultName])
  return stdout.trim() || null
}

export async function confirmUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'> {
  const script = `try
set choice to button returned of (display dialog "This document has unsaved changes." buttons {"Cancel", "Discard", "Save"} default button "Save" with title "Tinytext")
return choice
on error number -128
return "Cancel"
end try`
  const { stdout } = await execFileAsync('osascript', ['-e', script])
  const choice = stdout.trim()
  if (choice === 'Save') return 'save'
  if (choice === 'Discard') return 'discard'
  return 'cancel'
}
