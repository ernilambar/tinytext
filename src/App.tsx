import { useState } from 'react'
import { useGpuixRequired } from '@gpuix/react'
import { chooseSavePath, confirmUnsavedChanges, getDocumentName, readDocument, writeDocument } from './document-files'
import { getDocumentStats } from './document-stats'

const initialText = `Hello, world.\n\nWelcome to Tinytext, a small native text editor for macOS.\n\nStart typing here.`

function ToolbarButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <div
      role="button"
      aria-label={label}
      tabIndex={0}
      onClick={onClick}
      onKeyDown={(event) => {
        if (event.key === 'enter' || event.key === 'space') onClick()
      }}
      style={{
        display: 'flex',
        alignItems: 'center',
        height: 32,
        paddingLeft: 12,
        paddingRight: 12,
        borderRadius: 5,
        borderWidth: 1,
        borderColor: '#dedbd3',
        backgroundColor: '#fbfaf7',
        color: '#55554e',
        cursor: 'pointer',
        hover: { backgroundColor: '#efede7' },
      }}
    >
      <text style={{ color: '#55554e', fontSize: 12 }}>{label}</text>
    </div>
  )
}

export function App() {
  const renderer = useGpuixRequired()
  const [text, setText] = useState(initialText)
  const [savedText, setSavedText] = useState(initialText)
  const [filePath, setFilePath] = useState<string | null>(null)
  const [status, setStatus] = useState('Ready')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const { words, characters } = getDocumentStats(text)
  const isDirty = text !== savedText
  const title = filePath ? getDocumentName(filePath) : 'Untitled'

  const save = async () => {
    try {
      const destination = filePath ?? await chooseSavePath(filePath ? getDocumentName(filePath) : 'Untitled.txt')
      if (!destination) return false
      await writeDocument(destination, text)
      setFilePath(destination)
      setSavedText(text)
      setErrorMessage(null)
      setStatus('Saved')
      return true
    } catch (error) {
      setErrorMessage(`Could not save: ${error instanceof Error ? error.message : String(error)}`)
      setStatus('Save failed')
      return false
    }
  }

  const open = async () => {
    try {
      if (isDirty) {
        const choice = await confirmUnsavedChanges()
        if (choice === 'cancel') return
        if (choice === 'save' && !(await save())) return
      }

      const paths = await renderer.promptForPaths?.({ files: true, multiple: false, prompt: 'Open' })
      const selectedPath = paths?.[0]
      if (!selectedPath) return
      const contents = await readDocument(selectedPath)
      setText(contents)
      setSavedText(contents)
      setFilePath(selectedPath)
      setErrorMessage(null)
      setStatus('Opened')
    } catch (error) {
      setErrorMessage(`Could not open: ${error instanceof Error ? error.message : String(error)}`)
      setStatus('Open failed')
    }
  }

  return (
    <div
      onKeyDown={(event) => {
        if (!event.modifiers?.cmd) return
        if (event.key?.toLowerCase() === 's') {
          void save()
        } else if (event.key?.toLowerCase() === 'o') {
          void open()
        }
      }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        backgroundColor: '#f2f0eb',
        color: '#272722',
      }}
    >
      <div
        style={{
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          justifyContent: 'space-between',
          height: 58,
          paddingLeft: 28,
          paddingRight: 28,
          borderBottomWidth: 1,
          borderColor: '#dedbd3',
          backgroundColor: '#f7f6f2',
        }}
      >
        <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 11, minWidth: 0 }}>
          <text style={{ color: '#393a32', fontSize: 15, fontWeight: 600 }}>tinytext</text>
          <div style={{ width: 1, height: 17, backgroundColor: '#d8d5cd' }} />
          <text style={{ color: '#77776f', fontSize: 13 }}>{title}{isDirty ? ' •' : ''}</text>
        </div>
        <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 8 }}>
          <ToolbarButton label="Open" onClick={() => void open()} />
          <ToolbarButton label="Save" onClick={() => void save()} />
          <text style={{ color: isDirty ? '#a66f37' : '#77776f', fontSize: 12 }}>{status}</text>
        </div>
      </div>

      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          flexGrow: 1,
          alignItems: 'center',
          paddingTop: 43,
          paddingBottom: 22,
          paddingLeft: 28,
          paddingRight: 28,
        }}
      >
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            flexGrow: 1,
            width: '100%',
            maxWidth: 760,
            backgroundColor: '#fbfaf7',
            borderWidth: 1,
            borderColor: '#e5e2da',
            borderRadius: 5,
            paddingTop: 39,
            paddingBottom: 24,
            paddingLeft: 50,
            paddingRight: 50,
          }}
        >
          <text style={{ color: '#9a978e', fontSize: 10, fontWeight: 600 }}>{title.toUpperCase()}</text>
          <div style={{ height: 24 }} />
          <textarea
            value={text}
            onChange={(event) => {
              setText(event.value ?? '')
              setErrorMessage(null)
              setStatus('Editing')
            }}
            autoFocus
            style={{
              flexGrow: 1,
              minHeight: 0,
              color: '#34342e',
              backgroundColor: '#fbfaf7',
              fontSize: 16,
              lineHeight: 28,
            }}
          />
          {errorMessage ? (
            <text style={{ color: '#a54c3c', fontSize: 12 }}>{errorMessage}</text>
          ) : null}
          <div
            style={{
              display: 'flex',
              flexDirection: 'row',
              justifyContent: 'space-between',
              paddingTop: 16,
              borderTopWidth: 1,
              borderColor: '#ece9e2',
            }}
          >
            <text style={{ color: '#9a978e', fontSize: 11 }}>Plain text</text>
            <text style={{ color: '#9a978e', fontSize: 11 }}>
              {words} {words === 1 ? 'word' : 'words'} · {characters} characters
            </text>
          </div>
        </div>
      </div>
    </div>
  )
}
