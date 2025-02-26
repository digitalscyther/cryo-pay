import React, { useState, useEffect } from 'react';
import { Container, Button, Alert, Spinner, Form, Row, Col } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../../utils';

function Webhooks() {
    const [webhooks, setWebhooks] = useState();
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [creating, setCreating] = useState(false);
    const [newUrl, setNewUrl] = useState('');

    useEffect(() => {
        const fetchWebhooks = async () => {
            try {
                const response = await axios.get(apiUrl('/user/webhook'), { withCredentials: true });
                setWebhooks(response.data);
            } catch (err) {
                setError('Failed to load webhooks.');
            } finally {
                setLoading(false);
            }
        };
        fetchWebhooks();
    }, []);

    const handleCreate = async (event) => {
        event.preventDefault();
        setCreating(true);
        setError(null);
        try {
            const response = await axios.post(
                apiUrl('/user/webhook'),
                { url: newUrl },
                { withCredentials: true }
            );
            setWebhooks([response.data, ...webhooks]);
            setNewUrl('');
        } catch (err) {
            if (err.response) {
                if (err.response.status === 400) {
                    setError(err.response.data.message);
                } else if (err.response.status === 429) {
                    setError('Too many requests. Please try again later.');
                } else {
                    setError('Failed to create webhook.');
                }
            } else {
                setError('Failed to create webhook.');
            }
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async (id) => {
        setError(null);
        try {
            await axios.delete(apiUrl(`/user/webhook/${id}`), { withCredentials: true });
            setWebhooks(webhooks.filter((hook) => hook.id !== id));
        } catch (err) {
            setError('Failed to delete webhook.');
        }
    };

    if (loading) return <div><Spinner animation="border" /></div>;

    return (
        <Container>
            <h3 className="text-dark">Webhooks</h3>
            <p className="text-dark">
                Manage your webhook URLs.
            </p>

            {error && <Alert variant="danger">{error}</Alert>}

            <Form onSubmit={handleCreate} className="mb-3">
                <Row className="g-2">
                    <Col xs={9}>
                        <Form.Control
                            type="url"
                            placeholder="Enter webhook URL"
                            value={newUrl}
                            onChange={(e) => setNewUrl(e.target.value)}
                            required
                            disabled={creating}
                        />
                    </Col>
                    <Col xs={3}>
                        <Button type="submit" variant="outline-dark" disabled={creating}>
                            {creating ? 'Adding...' : 'Add'}
                        </Button>
                    </Col>
                </Row>
            </Form>

            {webhooks.length === 0 ? (
                <div>No webhooks available.</div>
            ) : (
                <ul className="list-unstyled">
                    {webhooks.map((hook) => (
                        <li
                            key={hook.id}
                            className="d-flex align-items-center mb-1"
                            style={{ maxWidth: '500px' }}
                        >
                            <Button
                                variant="danger"
                                size="sm"
                                className="me-2"
                                onClick={() => handleDelete(hook.id)}
                            >
                                X
                            </Button>
                            <span className="text-truncate">
                                {hook.url}
                            </span>
                        </li>
                    ))}
                </ul>
            )}
        </Container>
    );
}

export default Webhooks;