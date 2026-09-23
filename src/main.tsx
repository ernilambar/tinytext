import { existsSync } from 'node:fs'
import { createRequire } from 'node:module'
import { dirname, join } from 'node:path'

const require = createRequire(import.meta.url)
const packagedBinding = join(dirname(process.execPath), 'gpuix-native.darwin-arm64.node')
process.env.NAPI_RS_NATIVE_LIBRARY_PATH = existsSync(packagedBinding)
  ? packagedBinding
  : require.resolve('@gpuix/native-darwin-arm64')

const [{ render }, { App }] = await Promise.all([
  import('@gpuix/react'),
  import('./App'),
])

render(<App />, { title: 'Tinytext', width: 900, height: 680 })
