/**
 * Tests for error handling utilities
 */

import { describe, it, expect } from 'vitest';
import { getErrorMessage, formatErrorForDisplay } from './errorUtils';

describe('errorUtils', () => {
  describe('getErrorMessage', () => {
    it('should extract message from Error instance', () => {
      const error = new Error('Test error message');
      expect(getErrorMessage(error)).toBe('Test error message');
    });

    it('should extract message from error-like object', () => {
      const error = { message: 'Custom error' };
      expect(getErrorMessage(error)).toBe('Custom error');
    });

    it('should handle nested error object (backend format)', () => {
      const error = {
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Profile not found',
        },
      };
      // This extracts the error.message property which itself is an object
      // In this case, it will stringify the nested object
      const result = getErrorMessage(error);
      expect(result).toContain('code');
      expect(result).toContain('INTERNAL_ERROR');
    });

    it('should handle string errors', () => {
      const error = 'Simple string error';
      expect(getErrorMessage(error)).toBe('Simple string error');
    });

    it('should handle null/undefined with fallback', () => {
      expect(getErrorMessage(null)).toBe('An error occurred');
      expect(getErrorMessage(undefined)).toBe('An error occurred');
      expect(getErrorMessage(null, 'Custom fallback')).toBe('Custom fallback');
    });

    it('should handle objects without message by stringifying', () => {
      const error = { code: 404, status: 'not found' };
      const result = getErrorMessage(error);
      expect(result).toContain('404');
      expect(result).toContain('not found');
    });

    it('should handle primitive values', () => {
      expect(getErrorMessage(123)).toBe('123');
      expect(getErrorMessage(true)).toBe('true');
      expect(getErrorMessage(false)).toBe('false');
    });

    it('should handle error with object message (the [object Object] bug)', () => {
      // This simulates the bug we fixed where err.message is an object
      const error = {
        message: {
          code: 'INTERNAL_ERROR',
          text: 'Something went wrong',
        },
      };

      const result = getErrorMessage(error);
      // Should stringify the object instead of getting [object Object]
      expect(result).toContain('code');
      expect(result).toContain('INTERNAL_ERROR');
      expect(result).not.toBe('[object Object]');
    });
  });

  describe('formatErrorForDisplay', () => {
    it('should format error with context', () => {
      const error = new Error('Connection timeout');
      const result = formatErrorForDisplay(error, 'Failed to connect');
      expect(result).toBe('Failed to connect: Connection timeout');
    });

    it('should avoid duplicate context', () => {
      const error = new Error('Failed to connect: timeout');
      const result = formatErrorForDisplay(error, 'Failed to connect');
      // Should not duplicate the context
      expect(result).toBe('Failed to connect: timeout');
    });

    it('should work without context', () => {
      const error = new Error('Something went wrong');
      const result = formatErrorForDisplay(error);
      expect(result).toBe('Something went wrong');
    });

    it('should handle non-Error objects with context', () => {
      const error = { message: 'Network error' };
      const result = formatErrorForDisplay(error, 'Failed to fetch');
      expect(result).toBe('Failed to fetch: Network error');
    });
  });

  describe('Backend error format handling', () => {
    it('should properly extract message from backend error response', () => {
      // Simulate what the backend returns
      const backendError = {
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Profile not found',
        },
      };

      // The API client should extract error.message
      // In our case, we're testing that our utility doesn't break with nested structures
      const result = getErrorMessage(backendError.error.message);
      expect(result).toBe('Profile not found');
    });

    it('should handle when backend error object is passed directly', () => {
      const errorObj = {
        code: 'NOT_FOUND',
        message: 'Resource does not exist',
      };

      // If the error object itself is passed, we should get something useful
      const result = getErrorMessage(errorObj);
      expect(result).toContain('Resource does not exist');
    });
  });
});
