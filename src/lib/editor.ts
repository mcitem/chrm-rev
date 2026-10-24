import { loader } from '@guolao/vue-monaco-editor'
import * as monaco from 'monaco-editor/editor/editor.api'
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker'
import JsonWorker from 'monaco-editor/language/json/json.worker.js?worker'
import 'monaco-editor/language/json/monaco.contribution'

globalThis.MonacoEnvironment = {
  getWorker(_, label) {
    return label === 'json' ? new JsonWorker() : new EditorWorker()
  },
}
loader.config({ monaco })
