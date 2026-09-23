import { useState } from 'react'
import { getDocumentStats } from './document-stats'

const initialText = `Hello, world.\n\nWelcome to Tinytext, a small native text editor for macOS.\n\nStart typing here.`

export function App() {
  const [text, setText] = useState(initialText)
  const { words, characters } = getDocumentStats(text)

  return (
    <div
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
        <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 11 }}>
          <text style={{ color: '#393a32', fontSize: 15, fontWeight: 600 }}>tinytext</text>
          <div style={{ width: 1, height: 17, backgroundColor: '#d8d5cd' }} />
          <text style={{ color: '#77776f', fontSize: 13 }}>Untitled</text>
        </div>
        <div style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 7 }}>
          <div
            style={{
              width: 6,
              height: 6,
              borderRadius: 3,
              backgroundColor: text === initialText ? '#8c9b81' : '#c28b55',
            }}
          />
          <text style={{ color: '#77776f', fontSize: 12 }}>
            {text === initialText ? 'Ready' : 'Editing'}
          </text>
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
          <text style={{ color: '#9a978e', fontSize: 10, fontWeight: 600 }}>UNTITLED DOCUMENT</text>
          <div style={{ height: 24 }} />
          <textarea
            value={text}
            onChange={(event) => setText(event.value ?? '')}
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

