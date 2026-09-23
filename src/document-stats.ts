export function getDocumentStats(text: string) {
  const trimmedText = text.trim()

  return {
    words: trimmedText ? trimmedText.split(/\s+/).length : 0,
    characters: text.length,
  }
}
