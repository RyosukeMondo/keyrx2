/**
 * ESLint custom rule: test-naming-convention
 *
 * Enforces test file naming conventions based on test type:
 * - Unit tests: *.test.ts or *.test.tsx
 * - Integration tests: *.integration.test.ts or *.integration.test.tsx
 * - E2E tests: *.e2e.ts
 *
 * This rule checks for common mistakes like:
 * - Using .spec.* instead of .test.*
 * - Mixing naming conventions (e.g., .integration.spec.tsx)
 * - Missing test type indicator for complex tests
 */

module.exports = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce test file naming conventions',
      category: 'Best Practices',
      recommended: true,
    },
    messages: {
      invalidNaming: 'Test files should use naming convention: {{expected}}',
      useTestNotSpec: 'Use ".test.*" instead of ".spec.*" for test files',
      needsTypeIndicator: 'Integration/E2E tests should include type indicator (.integration.test.* or .e2e.*)',
    },
    schema: [],
  },

  create(context) {
    return {
      Program(node) {
        const filename = context.getFilename();

        // Skip non-test files and node_modules
        if (!filename.includes('test') && !filename.includes('spec') && !filename.includes('e2e')) {
          return;
        }

        if (filename.includes('node_modules')) {
          return;
        }

        // Check for .spec.* pattern (should be .test.*)
        if (filename.match(/\.spec\.(ts|tsx|js|jsx)$/)) {
          context.report({
            node,
            messageId: 'useTestNotSpec',
            loc: { line: 1, column: 0 },
          });
          return;
        }

        // Valid patterns
        const validPatterns = [
          /\.test\.(ts|tsx)$/,              // Unit tests
          /\.integration\.test\.(ts|tsx)$/, // Integration tests
          /\.e2e\.(ts|tsx)$/,               // E2E tests
        ];

        const isValid = validPatterns.some(pattern => pattern.test(filename));

        if (!isValid && (filename.includes('test') || filename.includes('spec'))) {
          // Determine expected pattern based on file location
          let expected = '*.test.ts or *.test.tsx (unit), *.integration.test.ts (integration), *.e2e.ts (E2E)';

          if (filename.includes('e2e')) {
            expected = '*.e2e.ts or *.e2e.tsx';
          } else if (filename.includes('integration')) {
            expected = '*.integration.test.ts or *.integration.test.tsx';
          }

          context.report({
            node,
            messageId: 'invalidNaming',
            data: { expected },
            loc: { line: 1, column: 0 },
          });
        }
      },
    };
  },
};
