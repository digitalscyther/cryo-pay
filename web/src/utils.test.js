jest.mock('axios');

import {
    apiUrl,
    getProjectName,
    getNetwork,
    getBlockchainIconPath,
    getSubscriptionInfo,
    NETWORKS,
    sortNetworkItems,
} from './utils';

describe('apiUrl', () => {
    const originalEnv = process.env;

    beforeEach(() => {
        process.env = { ...originalEnv };
    });

    afterAll(() => {
        process.env = originalEnv;
    });

    it('prepends BASE_API_URL to path', () => {
        process.env.REACT_APP_BASE_API_URL = '/api';
        expect(apiUrl('/foo')).toBe('/api/foo');
    });
});

describe('getProjectName', () => {
    const originalEnv = process.env;

    beforeEach(() => {
        process.env = { ...originalEnv };
    });

    afterAll(() => {
        process.env = originalEnv;
    });

    it('returns env var when set', () => {
        process.env.REACT_APP_PROJECT_NAME = 'TestProject';
        expect(getProjectName()).toBe('TestProject');
    });

    it('returns fallback when not set', () => {
        delete process.env.REACT_APP_PROJECT_NAME;
        expect(getProjectName()).toBe('MyApp');
    });
});

describe('getNetwork', () => {
    it('returns network for known ID', () => {
        const network = getNetwork(10);
        expect(network).toBeDefined();
        expect(network.name).toBe('Optimism');
    });

    it('returns undefined for unknown ID', () => {
        expect(getNetwork(999)).toBeUndefined();
    });
});

describe('getBlockchainIconPath', () => {
    it('returns optimism icon', () => {
        expect(getBlockchainIconPath('optimism')).toBe('/files/optimism-ethereum-op-logo.svg');
    });

    it('returns arbitrum icon', () => {
        expect(getBlockchainIconPath('Arbitrum-One')).toBe('/files/arbitrum-arb-logo.svg');
    });

    it('returns default icon for unknown', () => {
        expect(getBlockchainIconPath('unknown')).toBe('/files/optimism-sepolia.svg');
    });
});

describe('getSubscriptionInfo', () => {
    it('returns info for private_invoices', () => {
        expect(getSubscriptionInfo('private_invoices')).toContain('private invoices');
    });

    it('returns info for unlimited_invoices', () => {
        expect(getSubscriptionInfo('unlimited_invoices')).toContain('unlimited');
    });

    it('returns fallback for unknown key', () => {
        expect(getSubscriptionInfo('nonexistent')).toBe('No information available for this subscription.');
    });
});

describe('NETWORKS', () => {
    it('contains expected network IDs', () => {
        expect(NETWORKS[10]).toBeDefined();
        expect(NETWORKS[42161]).toBeDefined();
        expect(NETWORKS[11155420]).toBeDefined();
    });
});

describe('sortNetworkItems', () => {
    it('sorts items by name', () => {
        const items = [
            { name: 'Optimism' },
            { name: 'Arbitrum' },
            { name: 'Zeta' },
        ];
        const sorted = [...items].sort(sortNetworkItems);
        expect(sorted[0].name).toBe('Arbitrum');
        expect(sorted[1].name).toBe('Optimism');
        expect(sorted[2].name).toBe('Zeta');
    });
});
