import { defineConfig } from 'vite-plus'

export default defineConfig({
  lint: {
    options: {
      typeAware: true,
      typeCheck: true,
    },
    plugins: [
      'eslint',
      'typescript',
      'unicorn',
      'react',
      'react-perf',
      'oxc',
      'import',
      'jsx-a11y',
      'promise',
    ],
    ignorePatterns: [
      'packages/app/src/components/ui/*.tsx',
      'packages/app/src/hooks/use-mobile.ts',
      'packages/app/src/lib/utils.ts',
    ],
  },
  fmt: {
    semi: false,
    singleQuote: true,
    sortImports: true,
    sortPackageJson: true,
    ignorePatterns: [
      'packages/app/src/components/ui/*.tsx',
      'packages/app/src/hooks/use-mobile.ts',
      'packages/app/src/lib/utils.ts',
    ],
  },
  staged: {
    '*': 'vp check --fix',
  },
})
