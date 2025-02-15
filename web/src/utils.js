import axios from 'axios';

export const NETWORKS = {
    11155420: {
        "id": 11155420,
        "order": 1,
        "name": "Sepolia-Optimism (test)"
    },
    10: {
        "id": 10,
        "order": 2,
        "name": "Optimism"
    },
    42161: {
        "id": 42161,
        "order": 3,
        "name": "Arbitrum-One"
    }
};

export function apiUrl(path) {
    return `${process.env.REACT_APP_BASE_API_URL}${path}`
}

export function getProjectName() {
    return process.env.REACT_APP_PROJECT_NAME || "MyApp"
}

export function getContacts() {
    const text = process.env.REACT_APP_CONTACTS || '{"email":"foo@bar.baz","telegram":"foo","linkedin":"foo"}';
    return JSON.parse(text)
}

export function getSendMessageUrl() {
    return process.env.REACT_APP_SEND_MESSAGE_URL
}

export function getProjectGitHubUrl() {
    return process.env.REACT_APP_PROJECT_GITHUB_URL || "https://github.com/foo/bar"
}

export function getNetwork(networkId) {
    return NETWORKS[networkId]
}

export async function getBlockchainInfo() {
    return await axios.get(apiUrl("/blockchain/info"))
}

export async function getInvoice(invoice_id) {
    return await axios.get(
        apiUrl(`/payment/invoice/${invoice_id}?with_own=true`),
        {withCredentials: true}
    );
}

export const getFullUrl = (path = '') => {
    const baseUrl = window.location.origin;
    return path ? new URL(path, baseUrl).href : baseUrl;
};

export const getAvailableNetworks = async () => {
    try {
        let response = await getBlockchainInfo();
        return response.data.networks.map((item) => item.name ).sort()

    } catch (err) {
        console.error(err);
        return [];
    }
}

export const getBlockchainIconPath = (blockchain) => {
    const iconMap = {
        'arbitrum-one': '/files/arbitrum-arb-logo.svg',
        'optimism': '/files/optimism-ethereum-op-logo.svg'
    };

    return iconMap[blockchain.toLowerCase()] || '/files/optimism-sepolia.svg';
};

export const getSubscriptionInfo = (key) => {
    const info = {
        'private_invoices': 'Enables the creation of private invoices that are excluded from the public invoice list. Only the creator can view these invoices within the interface, but they remain accessible to anyone with a direct link.',
        'unlimited_invoices': 'Removes the invoice creation limit, allowing users to generate an unlimited number of invoices. This feature is particularly beneficial for high-volume users who exceed the restrictions imposed on free accounts.',
    };
    return info[key] || 'No information available for this subscription.';
}

export const sortNetworkItems = (a, b) => {
    return (a, b) => a.name.localeCompare(b.name)
}

export const exampleEthereumAddressHere = "0xYourEthereumAddressHere";
