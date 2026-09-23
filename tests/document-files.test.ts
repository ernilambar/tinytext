import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { getDocumentName, readDocument, writeDocument } from '../src/document-files'

let directory: string

describe('document file operations', () => {
  beforeEach(async () => {
    directory = await mkdtemp(join(tmpdir(), 'tinytext-'))
  })

  afterEach(async () => {
    await rm(directory, { recursive: true, force: true })
  })

  test('writes and reads UTF-8 text without changing newlines', async () => {
    const path = join(directory, 'notes.txt')
    const content = 'Hello, world.\nこんにちは 🌿\n'

    await writeDocument(path, content)

    expect(await readDocument(path)).toBe(content)
  })

  test('rejects when a file cannot be read', async () => {
    await expect(readDocument(join(directory, 'missing.txt'))).rejects.toThrow()
  })

  test('returns the final path component as the document name', () => {
    expect(getDocumentName(join(directory, 'notes.txt'))).toBe('notes.txt')
  })
})
