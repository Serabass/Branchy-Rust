import { useCallback, useEffect, useRef, useState } from 'react'
import Editor, { type OnMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'
import { branchyLanguage, branchyLanguageId } from './branchyLanguage'

let monacoGlobal: typeof import('monaco-editor') | null = null

type ExampleItem = { id: string; name: string; source: string }

type TraceSpan = {
  start_line: number
  start_column: number
  end_line: number
  end_column: number
}

type RunOk = { result: string; trace?: TraceSpan[] }
type RunErr = {
  error: string
  line?: number
  column?: number
  end_line?: number
  end_column?: number
}

type FormatOk = { formatted: string }
type FormatErr = RunErr

const defaultSource = `[
  greet :who { :who = [ world; human; ]; };
]`

export default function App() {
  const [examples, setExamples] = useState<ExampleItem[]>([])
  const [source, setSource] = useState(defaultSource)
  const [selectedExampleId, setSelectedExampleId] = useState<string>('')
  const [input, setInput] = useState('')
  const [seedInput, setSeedInput] = useState('')
  const [result, setResult] = useState<
    | { ok: string; trace?: TraceSpan[] }
    | { error: string; line?: number; column?: number; end_line?: number; end_column?: number }
    | null
  >(null)
  const [loading, setLoading] = useState(false)
  const [formatLoading, setFormatLoading] = useState(false)
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const decorationIdsRef = useRef<string[]>([])

  useEffect(() => {
    fetch('/api/examples')
      .then((r) => r.json())
      .then((data: ExampleItem[]) => setExamples(Array.isArray(data) ? data : []))
      .catch(() => setExamples([]))
  }, [])

  const applyDecorations = useCallback(
    (trace: TraceSpan[] | undefined, errorSpan: RunErr | undefined) => {
      const ed = editorRef.current
      if (!ed) return
      const monaco = monacoGlobal
      if (!monaco) return
      const model = ed.getModel()
      if (!model) return
      const ids = ed.deltaDecorations(decorationIdsRef.current, [])
      decorationIdsRef.current = ids
      if (errorSpan?.line != null && errorSpan?.column != null) {
        const endLn = errorSpan.end_line ?? errorSpan.line
        const endCol =
          errorSpan.end_column != null ? errorSpan.end_column : errorSpan.column + 1
        const decorations: editor.IModelDeltaDecoration[] = [
          {
            range: new monaco.Range(errorSpan.line, errorSpan.column, endLn, endCol),
            options: {
              isWholeLine: false,
              className: 'error-highlight',
              marginClassName: 'error-margin',
            },
          },
        ]
        decorationIdsRef.current = ed.deltaDecorations(decorationIdsRef.current, decorations)
        return
      }
      if (!trace?.length) return
      const decorations: editor.IModelDeltaDecoration[] = trace.map((s) => ({
        range: new monaco.Range(
          s.start_line,
          s.start_column,
          s.end_line,
          s.end_column + 1
        ),
        options: {
          isWholeLine: false,
          className: 'trace-highlight',
          marginClassName: 'trace-margin',
        },
      }))
      decorationIdsRef.current = ed.deltaDecorations(decorationIdsRef.current, decorations)
    },
    []
  )

  useEffect(() => {
    if (!result) {
      applyDecorations(undefined, undefined)
      return
    }
    if ('ok' in result) {
      applyDecorations(result.trace, undefined)
    } else {
      applyDecorations(undefined, result)
    }
  }, [result, applyDecorations])

  function selectExample(id: string) {
    setSelectedExampleId(id)
    const ex = examples.find((e) => e.id === id)
    if (ex) setSource(ex.source)
  }

  const handleEditorMount: OnMount = useCallback((editorInstance, monaco) => {
    editorRef.current = editorInstance
    monacoGlobal = monaco
    ;(window as unknown as { monaco: typeof monaco }).monaco = monaco
    try {
      monaco.languages.register({ id: branchyLanguageId })
      monaco.languages.setMonarchTokensProvider(branchyLanguageId, branchyLanguage)
    } catch {
      // already registered
    }
  }, [])

  async function handleRun() {
    setLoading(true)
    setResult(null)
    try {
      const body: { source: string; input?: string; seed?: number } = { source }
      if (input.trim()) body.input = input.trim()
      const seedNum = seedInput.trim() ? parseInt(seedInput.trim(), 10) : undefined
      if (seedNum !== undefined && !Number.isNaN(seedNum) && seedNum >= 0) body.seed = seedNum
      const res = await fetch('/api/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })
      const data = (await res.json()) as RunOk | RunErr
      if (!res.ok) {
        const errMsg = 'error' in data ? data.error : res.statusText
        setResult({
          error: errMsg,
          line: 'line' in data ? data.line : undefined,
          column: 'column' in data ? data.column : undefined,
          end_line: 'end_line' in data ? data.end_line : undefined,
          end_column: 'end_column' in data ? data.end_column : undefined,
        })
        return
      }
      setResult({
        ok: 'result' in data ? data.result ?? '' : '',
        trace: 'trace' in data ? data.trace : undefined,
      })
    } catch (e) {
      setResult({ error: e instanceof Error ? e.message : String(e) })
    } finally {
      setLoading(false)
    }
  }

  async function handleFormat() {
    setFormatLoading(true)
    setResult(null)
    try {
      const res = await fetch('/api/format', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source }),
      })
      const data = (await res.json()) as FormatOk | FormatErr
      if (!res.ok) {
        const errMsg = 'error' in data ? data.error : res.statusText
        setResult({
          error: errMsg,
          line: 'line' in data ? data.line : undefined,
          column: 'column' in data ? data.column : undefined,
          end_line: 'end_line' in data ? data.end_line : undefined,
          end_column: 'end_column' in data ? data.end_column : undefined,
        })
        return
      }
      if ('formatted' in data) {
        setSource(data.formatted)
      }
    } catch (e) {
      setResult({ error: e instanceof Error ? e.message : String(e) })
    } finally {
      setFormatLoading(false)
    }
  }

  return (
    <>
      <h1>Branchy</h1>
      <div className="field">
        <label htmlFor="example">Пример</label>
        <select
          id="example"
          value={selectedExampleId}
          onChange={(e) => selectExample(e.target.value)}
          className="example-select"
        >
          <option value="">— свой исходник —</option>
          {examples.map((ex) => (
            <option key={ex.id} value={ex.id}>
              {ex.name}
            </option>
          ))}
        </select>
      </div>
      <div className="field">
        <label htmlFor="source">Исходник</label>
        <div className="editor-wrap">
          <Editor
            height="240px"
            language={branchyLanguageId}
            value={source}
            onChange={(v) => {
              setSource(v ?? '')
              setSelectedExampleId('')
            }}
            onMount={handleEditorMount}
            theme="vs-dark"
            options={{
              minimap: { enabled: false },
              fontSize: 14,
              scrollBeyondLastLine: false,
              wordWrap: 'on',
            }}
          />
        </div>
      </div>
      <div className="field">
        <label htmlFor="input">Вход для события (необязательно)</label>
        <input
          id="input"
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="например: start"
        />
      </div>
      <div className="field">
        <label htmlFor="seed">Seed (необязательно)</label>
        <input
          id="seed"
          type="number"
          min={0}
          step={1}
          value={seedInput}
          onChange={(e) => setSeedInput(e.target.value)}
          placeholder="пусто = случайный"
        />
      </div>
      <div className="button-row">
        <button onClick={handleRun} disabled={loading} className="run-btn">
          {loading ? 'Выполняю…' : 'Выполнить'}
        </button>
        <button
          onClick={handleFormat}
          disabled={formatLoading}
          className="format-btn"
          type="button"
        >
          {formatLoading ? 'Форматирую…' : 'Форматировать'}
        </button>
      </div>
      <div className="result-slot" aria-live="polite">
        {loading && !result && (
          <div className="result result-loading">Выполняю…</div>
        )}
        {result && (
          <div className={`result ${'ok' in result ? 'success' : 'error'}`}>
            {'ok' in result ? result.ok : result.error}
          </div>
        )}
      </div>
    </>
  )
}
