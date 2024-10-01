function api_url(path) {
    return `${process.env.REACT_APP_BASE_API_URL}${path}`
}

module.exports = {
    api_url
}
