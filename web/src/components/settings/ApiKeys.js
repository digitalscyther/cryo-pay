import React, {useState, useEffect} from 'react';
import {Container, Button, Alert, Spinner, Card} from 'react-bootstrap';
import axios from 'axios';
import {apiUrl} from '../../utils';
import LocalDate from "../common/LocalDate";
import {Link} from "react-router-dom";

function ApiKeys() {
    const [apiKeys, setApiKeys] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [creating, setCreating] = useState(false);
    const [newApiKey, setNewApiKey] = useState(null);

    useEffect(() => {
        const fetchApiKeys = async () => {
            try {
                const response = await axios.get(apiUrl('/user/api_key'), {withCredentials: true});
                setApiKeys(response.data);
            } catch (err) {
                setError('Failed to load API keys.');
            } finally {
                setLoading(false);
            }
        };
        fetchApiKeys();
    }, []);

    const handleCreate = async () => {
        setCreating(true);
        setError(null);
        setNewApiKey(null);
        try {
            const response = await axios.post(apiUrl('/user/api_key'), {}, {withCredentials: true});
            setNewApiKey(response.data.key);
            setApiKeys([response.data.instance, ...apiKeys]);
        } catch (err) {
            if (err.response && err.response.status === 400) {
                setError('You have exceeded the maximum number of API keys.');
            } else {
                setError('Failed to create API key.');
            }
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async (id) => {
        setError(null);
        try {
            await axios.delete(apiUrl(`/user/api_key/${id}`), {withCredentials: true});
            setApiKeys(apiKeys.filter((key) => key.id !== id));
        } catch (err) {
            setError('Failed to delete API key.');
        }
    };

    if (loading) return <div><Spinner animation="border"/></div>;

    return (
        <Container>
            <h3 className="text-dark">API Keys</h3>
            <div className="m-3"><Link to="/docs#api-endpoints">API Documentation</Link></div>
            <p className="text-dark">
                Manage your API keys for accessing the application programmatically.
            </p>

            {error && <Alert variant="danger">{error}</Alert>}
            {newApiKey && (
                <Alert variant="success">
                    <strong>New API Key:</strong> {newApiKey}
                    <br/>
                    <em>Make sure to save it now, as it won't be shown again!</em>
                </Alert>
            )}

            <div className="mb-3">
                <Button onClick={handleCreate} variant="outline-dark" disabled={creating}>
                    {creating ? 'Creating...' : 'Create New API Key'}
                </Button>
            </div>

            {apiKeys.length === 0 ? (
                <div className="text-light">No API keys available.</div>
            ) : (
                <div className="d-flex flex-wrap">
                    {apiKeys.map((key) => (
                        <Card
                            key={key.id}
                            className="m-3"
                            bg="dark"
                            text="light"
                            style={{width: '100%', maxWidth: '500px'}}
                        >
                            <Card.Body>
                                <Card.Title className="text-truncate">{key.id}</Card.Title>
                                <Card.Text>
                                    <strong>Created:</strong> <LocalDate date={key.created}/>
                                    <br/>
                                    <strong>Last
                                        Used:</strong> {key.last_used ? <LocalDate date={key.last_used}/> : 'Never'}
                                </Card.Text>
                                <Button
                                    variant="danger"
                                    size="sm"
                                    onClick={() => handleDelete(key.id)}
                                >
                                    Delete
                                </Button>
                            </Card.Body>
                        </Card>
                    ))}
                </div>
            )}
        </Container>
    );
}

export default ApiKeys;
