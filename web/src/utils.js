const SEPOLIA_OPTIMISM_NETWORK_ID = 11155420n;

function api_url(path) {
    return `${process.env.REACT_APP_BASE_API_URL}${path}`
}

module.exports = {
    SEPOLIA_OPTIMISM_NETWORK_ID,
    api_url,
}
