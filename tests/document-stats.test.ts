import { describe, expect, test } from 'bun:test'
import { getDocumentStats } from '../src/document-stats'

describe('getDocumentStats', () => {
  test('counts words separated by whitespace and characters', () => {
    expect(getDocumentStats('Hello, world.\nTinytext')).toEqual({
      words: 3,
      characters: 22,
    })
  })

  test('returns zero words for empty or whitespace-only text', () => {
    expect(getDocumentStats('')).toEqual({ words: 0, characters: 0 })
    expect(getDocumentStats('  \n\t ')).toEqual({ words: 0, characters: 5 })
  })

  test('counts consecutive whitespace as one word separator', () => {
    expect(getDocumentStats('  one   two\nthree  ')).toEqual({
      words: 3,
      characters: 19,
    })
  })
})
