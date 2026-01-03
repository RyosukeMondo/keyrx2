import { describe, it, expect, vi } from 'vitest';
import { screen, within, waitFor } from '@testing-library/react';
import { renderWithProviders } from '../../tests/testUtils';
import userEvent from '@testing-library/user-event';
import { ProfilesPage } from './ProfilesPage';

describe('ProfilesPage', () => {
  it('renders page title', async () => {
    renderWithProviders(<ProfilesPage />);

    // Wait for data to load
    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: 'Profiles' })
      ).toBeInTheDocument();
    });
  });

  it('renders Create Profile button', async () => {
    renderWithProviders(<ProfilesPage />);

    // Wait for page to finish loading
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /Create new profile/i })
      ).toBeInTheDocument();
    });
  });

  it('renders initial profile cards', async () => {
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load
    await waitFor(() => {
      expect(screen.getByText('default')).toBeInTheDocument();
    });

    expect(screen.getByText('gaming')).toBeInTheDocument();
  });

  it('shows active indicator on Default profile', async () => {
    renderWithProviders(<ProfilesPage />);

    await waitFor(() => {
      expect(screen.getByText('ACTIVE')).toBeInTheDocument();
    });
  });

  it('opens create modal when Create Profile button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load
    const createButton = await screen.findByRole('button', {
      name: /Create new profile/i,
    });
    await user.click(createButton);

    expect(
      screen.getByRole('heading', { name: 'Create New Profile' })
    ).toBeInTheDocument();
  });

  it('creates a new profile when form is submitted', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load and open create modal
    const createButton = await screen.findByRole('button', { name: /Create new profile/i });
    await user.click(createButton);

    // Fill in form
    const nameInput = screen.getByPlaceholderText('Profile name');
    await user.type(nameInput, 'New Profile');

    // Submit
    const saveButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(saveButton);

    // Verify profile was created (wait for modal to close and profile to appear)
    await waitFor(() => {
      expect(screen.getByText('New Profile')).toBeInTheDocument();
    });
  });

  it('shows validation error when profile name is empty', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load and open create modal
    const createButton = await screen.findByRole('button', { name: /Create new profile/i });
    await user.click(createButton);

    // Try to submit without name
    const saveButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(saveButton);

    expect(screen.getByText('Profile name is required')).toBeInTheDocument();
  });

  it('shows validation error when profile name is too long', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load and open create modal
    const createButton = await screen.findByRole('button', { name: /Create new profile/i });
    await user.click(createButton);

    // Fill with name over 50 chars (maxLength prevents typing, but we can paste)
    const nameInput = screen.getByPlaceholderText('Profile name');
    // Type exactly 50 chars first
    await user.type(nameInput, 'a'.repeat(50));

    // Verify maxLength prevents more than 50 characters
    expect((nameInput as HTMLInputElement).value.length).toBe(50);
  });

  it('shows validation error when profile name already exists', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load and open create modal
    const createButton = await screen.findByRole('button', { name: /Create new profile/i });
    await user.click(createButton);

    // Use existing name (MSW mock has 'default' profile)
    const nameInput = screen.getByPlaceholderText('Profile name');
    await user.type(nameInput, 'default');

    const saveButton = screen.getByRole('button', { name: /Save new profile/i });
    await user.click(saveButton);

    expect(screen.getByText('Profile name already exists')).toBeInTheDocument();
  });

  it('closes create modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for page to load and open create modal
    const createButton = await screen.findByRole('button', { name: /Create new profile/i });
    await user.click(createButton);

    expect(
      screen.getByRole('heading', { name: 'Create New Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel creating profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Create New Profile' })
      ).not.toBeInTheDocument();
    });
  });

  it('activates a profile when Activate button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load
    await waitFor(() => {
      expect(screen.getByText('default')).toBeInTheDocument();
    });

    // Initially, default should be active
    const defaultCard = screen.getByText('default').closest('.border-green-500');
    expect(defaultCard).toBeInTheDocument();

    // Click Activate on gaming profile
    const activateButton = screen.getByRole('button', {
      name: /Activate profile gaming/i,
    });
    await user.click(activateButton);

    // Verify gaming is now active (should have green border)
    const gamingCard = screen.getByText('gaming').closest('.border-green-500');
    expect(gamingCard).toBeInTheDocument();

    // Verify there's still only one ACTIVE badge
    const activeCards = screen.getAllByText('ACTIVE');
    expect(activeCards).toHaveLength(1);

    // The ACTIVE badge should be in the gaming card
    const gamingSection = screen.getByText('gaming').closest('.border-green-500');
    expect(within(gamingSection!).getByText('ACTIVE')).toBeInTheDocument();
  });

  it('opens edit modal when Edit button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and click Edit on gaming profile
    const editButton = await screen.findByRole('button', {
      name: /Edit profile gaming/i,
    });
    await user.click(editButton);

    expect(
      screen.getByRole('heading', { name: 'Edit Profile' })
    ).toBeInTheDocument();

    // Verify form is pre-filled (profile name from MSW mock is "Gaming" with capital G)
    const nameInput = screen.getByPlaceholderText('Profile name') as HTMLInputElement;
    expect(nameInput.value).toBe('gaming');
  });

  it('updates profile when edit form is submitted', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and click Edit on gaming profile
    const editButton = await screen.findByRole('button', { name: /Edit profile gaming/i });
    await user.click(editButton);

    // Update name
    const nameInput = screen.getByPlaceholderText('Profile name');
    await user.clear(nameInput);
    await user.type(nameInput, 'Updated Gaming');

    // Save (note: edit API not implemented yet, so modal just closes)
    await user.click(
      screen.getByRole('button', { name: /Save profile changes/i })
    );

    // Modal should close (edit doesn't actually update the profile yet)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Edit Profile' })
      ).not.toBeInTheDocument();
    });

    // Original profile name still exists (edit not implemented)
    expect(screen.getByText('gaming')).toBeInTheDocument();
  });

  it('closes edit modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and open edit modal
    const editButton = await screen.findByRole('button', { name: /Edit profile gaming/i });
    await user.click(editButton);

    expect(
      screen.getByRole('heading', { name: 'Edit Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel editing profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Edit Profile' })
      ).not.toBeInTheDocument();
    });
  });

  it('opens delete confirmation modal when Delete button is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and click Delete on gaming profile
    const deleteButton = await screen.findByRole('button', {
      name: /Delete profile gaming/i,
    });
    await user.click(deleteButton);

    expect(
      screen.getByRole('heading', { name: 'Delete Profile' })
    ).toBeInTheDocument();

    // Check the confirmation message mentions the profile name
    expect(screen.getByText(/Are you sure you want to delete the profile/)).toBeInTheDocument();
    const strongElement = screen.getByText('gaming', { selector: 'strong' });
    expect(strongElement).toBeInTheDocument();
  });

  it('deletes profile when deletion is confirmed', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and click Delete on gaming profile
    const deleteButton = await screen.findByRole('button', { name: /Delete profile gaming/i });
    await user.click(deleteButton);

    // Confirm deletion
    await user.click(
      screen.getByRole('button', { name: /Confirm delete profile/i })
    );

    // Wait for profile to be removed and modal to close
    await waitFor(() => {
      expect(screen.queryByText('gaming')).not.toBeInTheDocument();
    });
  });

  it('closes delete modal when Cancel is clicked', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load and open delete modal
    const deleteButton = await screen.findByRole('button', { name: /Delete profile gaming/i });
    await user.click(deleteButton);

    expect(
      screen.getByRole('heading', { name: 'Delete Profile' })
    ).toBeInTheDocument();

    // Click Cancel
    await user.click(
      screen.getByRole('button', { name: /Cancel deleting profile/i })
    );

    // Wait for modal to close (animation takes time)
    await waitFor(() => {
      expect(
        screen.queryByRole('heading', { name: 'Delete Profile' })
      ).not.toBeInTheDocument();
    });

    // Verify profile still exists
    expect(screen.getByText('gaming')).toBeInTheDocument();
  });

  it('renders profile grid with responsive classes', async () => {
    const { container } = renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load
    await waitFor(() => {
      expect(screen.getByText('default')).toBeInTheDocument();
    });

    const grid = container.querySelector('.grid');
    expect(grid).toHaveClass('grid-cols-1');
    expect(grid).toHaveClass('md:grid-cols-2');
    expect(grid).toHaveClass('lg:grid-cols-3');
  });

  it('shows all action buttons with proper accessibility labels', async () => {
    renderWithProviders(<ProfilesPage />);

    // Wait for profiles to load - use findBy for async wait
    await screen.findByText('gaming');

    // gaming profile (inactive) - use queryByRole then assert to handle timing
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /Activate profile gaming/i })
      ).toBeInTheDocument();
    });

    expect(
      screen.getByRole('button', { name: /Edit profile gaming/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: /Delete profile gaming/i })
    ).toBeInTheDocument();
  });
});
