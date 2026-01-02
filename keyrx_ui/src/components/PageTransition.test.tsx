import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import { PageTransition } from './PageTransition';

describe('PageTransition', () => {
  it('renders children', () => {
    renderWithProviders(
      <PageTransition>
        <div>Page content</div>
      </PageTransition>
    );
    expect(screen.getByText('Page content')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = renderWithProviders(
      <PageTransition className="custom-class">
        <div>Page content</div>
      </PageTransition>
    );
    const wrapper = container.querySelector('.custom-class');
    expect(wrapper).toBeInTheDocument();
  });

  it('has aria-live attribute for accessibility', () => {
    const { container } = renderWithProviders(
      <PageTransition>
        <div>Page content</div>
      </PageTransition>
    );
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper).toHaveAttribute('aria-live', 'polite');
  });

  it('wraps content in motion.div for animation', () => {
    const { container } = renderWithProviders(
      <PageTransition>
        <div>Page content</div>
      </PageTransition>
    );
    // motion.div is rendered as a div
    expect(container.firstChild?.nodeName).toBe('DIV');
  });

  it('renders multiple children', () => {
    renderWithProviders(
      <PageTransition>
        <div>First child</div>
        <div>Second child</div>
      </PageTransition>
    );
    expect(screen.getByText('First child')).toBeInTheDocument();
    expect(screen.getByText('Second child')).toBeInTheDocument();
  });

  it('renders without className when not provided', () => {
    renderWithProviders(
      <PageTransition>
        <div>Page content</div>
      </PageTransition>
    );
    expect(screen.getByText('Page content')).toBeInTheDocument();
  });
});
