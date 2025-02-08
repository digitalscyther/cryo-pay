import React, { useState, useEffect } from 'react';
import { Container, Button, Alert, Spinner, Form, Row, Col } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../../utils';

function CallbackUrls() {
    const [callbackUrls, setCallbackUrls] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [creating, setCreating] = useState(false);
    const [newUrl, setNewUrl] = useState('');

    useEffect(() => {
        const fetchCallbackUrls = async () => {
            try {
                const response = await axios.get(apiUrl('/user/callback_url'), { withCredentials: true });
                setCallbackUrls(response.data);
            } catch (err) {
                setError('Failed to load callback URLs.');
            } finally {
                setLoading(false);
            }
        };
        fetchCallbackUrls();
    }, []);

    const handleCreate = async (event) => {
        event.preventDefault();
        setCreating(true);
        setError(null);
        try {
            const response = await axios.post(
                apiUrl('/user/callback_url'),
                { url: newUrl },
                { withCredentials: true }
            );
            setCallbackUrls([response.data, ...callbackUrls]);
            setNewUrl('');
        } catch (err) {
            if (err.response && err.response.status === 400) {
                setError('You have reached the maximum number of allowed callback URLs.');
            } else {
                setError('Failed to create callback URL.');
            }
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async (id) => {
        setError(null);
        try {
            await axios.delete(apiUrl(`/user/callback_url/${id}`), { withCredentials: true });
            setCallbackUrls(callbackUrls.filter((url) => url.id !== id));
        } catch (err) {
            setError('Failed to delete callback URL.');
        }
    };

    if (loading) return <div><Spinner animation="border" /></div>;

    return (
        <Container>
            <h3 className="text-dark">Callback URLs</h3>
            <p className="text-dark">
                Manage your callback URLs for whitelisting.
            </p>

            {error && <Alert variant="danger">{error}</Alert>}

            <Form onSubmit={handleCreate} className="mb-3">
                <Row className="g-2">
                    <Col xs={9}>
                        <Form.Control
                            type="url"
                            placeholder="Enter callback URL"
                            value={newUrl}
                            onChange={(e) => setNewUrl(e.target.value)}
                            required
                            disabled={creating}
                        />
                    </Col>
                    <Col xs={3}>
                        <Button type="submit" variant="outline-dark" disabled={creating} block>
                            {creating ? 'Adding...' : 'Add'}
                        </Button>
                    </Col>
                </Row>
            </Form>

            {callbackUrls.length === 0 ? (
                <div>No callback URLs available.</div>
            ) : (
                <ul className="list-unstyled">
                    {callbackUrls.map((url) => (
                        <li
                            key={url.id}
                            className="d-flex align-items-center mb-1"
                            style={{ maxWidth: '500px' }}
                        >
                            <Button
                                variant="danger"
                                size="sm"
                                className="me-2"
                                onClick={() => handleDelete(url.id)}
                            >
                                X
                            </Button>
                            <span className="text-truncate">
                                {url.url}
                            </span>
                        </li>
                    ))}
                </ul>
            )}
        </Container>
    );
}

export default CallbackUrls;
