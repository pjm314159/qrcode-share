import { describe, it, expect } from 'vitest';
import {
  validateLink,
  extractDomain,
  formatRemainingTime,
  generateChannelId,
  isWechatBrowser,
} from '../helpers';

describe('Helpers', () => {
  describe('validateLink', () => {
    it('should accept valid http URLs', () => {
      expect(validateLink('http://example.com')).toBe(true);
    });

    it('should accept valid https URLs', () => {
      expect(validateLink('https://example.com/path?q=1')).toBe(true);
    });

    it('should reject non-http protocols', () => {
      expect(validateLink('ftp://example.com')).toBe(false);
    });

    it('should reject invalid URLs', () => {
      expect(validateLink('not-a-url')).toBe(false);
    });

    it('should reject empty strings', () => {
      expect(validateLink('')).toBe(false);
    });
  });

  describe('extractDomain', () => {
    it('should extract domain from URL', () => {
      expect(extractDomain('https://example.com/path')).toBe('example.com');
    });

    it('should extract domain with subdomain', () => {
      expect(extractDomain('https://sub.example.com')).toBe('sub.example.com');
    });

    it('should return empty string for invalid URL', () => {
      expect(extractDomain('not-a-url')).toBe('');
    });
  });

  describe('formatRemainingTime', () => {
    it('should format seconds only', () => {
      expect(formatRemainingTime(45)).toBe('45s');
    });

    it('should format minutes and seconds', () => {
      expect(formatRemainingTime(125)).toBe('2m 5s');
    });

    it('should return Expired for zero', () => {
      expect(formatRemainingTime(0)).toBe('Expired');
    });

    it('should return Expired for negative', () => {
      expect(formatRemainingTime(-5)).toBe('Expired');
    });
  });

  describe('generateChannelId', () => {
    it('should generate 8-character ID', () => {
      const id = generateChannelId();
      expect(id).toHaveLength(8);
    });

    it('should generate unique IDs', () => {
      const ids = new Set(Array.from({ length: 100 }, () => generateChannelId()));
      expect(ids.size).toBeGreaterThan(90);
    });

    it('should only contain lowercase alphanumeric characters', () => {
      const id = generateChannelId();
      expect(id).toMatch(/^[a-z0-9]+$/);
    });
  });

  describe('isWechatBrowser', () => {
    it('should return false for non-WeChat browsers', () => {
      expect(isWechatBrowser()).toBe(false);
    });
  });
});
