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
    }
};

export function apiUrl(path) {
    return `${process.env.REACT_APP_BASE_API_URL}${path}`
}

export function getNetwork(networkId) {
    return NETWORKS[networkId]
}

export async function getBlockchainInfo() {
    return await axios.get(apiUrl("/blockchain/info"))
}
