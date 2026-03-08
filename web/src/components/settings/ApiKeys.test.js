import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import axios from 'axios';
import ApiKeys from './ApiKeys';

jest.mock('axios');

afterEach(() => jest.clearAllMocks());

function renderApiKeys() {
    return render(<MemoryRouter><ApiKeys /></MemoryRouter>);
}

test('shows loading spinner on mount', () => {
    axios.get.mockImplementation(() => new Promise(() => {}));
    const { container } = renderApiKeys();
    expect(container.querySelector('.spinner-border')).toBeInTheDocument();
});

test('renders api key IDs after successful fetch', async () => {
    axios.get.mockResolvedValue({
        data: [{ id: 'key-uuid-abc123', created: '2024-01-01', last_used: null }],
    });
    renderApiKeys();
    expect(await screen.findByText(/key-uuid-abc123/i)).toBeInTheDocument();
});

test('shows error alert on fetch failure', async () => {
    axios.get.mockRejectedValue(new Error('Network error'));
    renderApiKeys();
    expect(await screen.findByRole('alert')).toBeInTheDocument();
});
