import { render, screen } from '@testing-library/react';
import axios from 'axios';
import Webhooks from './Webhooks';

jest.mock('axios');

afterEach(() => jest.clearAllMocks());

test('shows loading spinner on mount', () => {
    axios.get.mockImplementation(() => new Promise(() => {}));
    const { container } = render(<Webhooks />);
    expect(container.querySelector('.spinner-border')).toBeInTheDocument();
});

test('renders webhook URLs after successful fetch', async () => {
    axios.get.mockResolvedValue({
        data: [{ id: 'hook-1', url: 'https://example.com/webhook' }],
    });
    render(<Webhooks />);
    expect(await screen.findByText(/example\.com\/webhook/i)).toBeInTheDocument();
});

test('shows error alert on fetch failure', async () => {
    axios.get.mockRejectedValue(new Error('Network error'));
    render(<Webhooks />);
    expect(await screen.findByRole('alert')).toBeInTheDocument();
});
