import React, {useState} from 'react';
import {Container, Row, Col, Card, Badge, Button, Modal, Form, Alert, Spinner} from 'react-bootstrap';
import axios from 'axios';
import LocalDate from "../common/LocalDate";
import {apiUrl} from '../../utils';

function Subscriptions({subscriptions}) {
    const [showModal, setShowModal] = useState(false);
    const [selectedSub, setSelectedSub] = useState(null);
    const [days, setDays] = useState(1);
    const [price, setPrice] = useState(null);
    const [loadingPrice, setLoadingPrice] = useState(false);
    const [error, setError] = useState(null);

    const formatSubscriptionKey = (key) => key.replace(/_/g, ' ').replace(/\b\w/g, (char) => char.toUpperCase());

    const handleOpenModal = (subKey) => {
        setSelectedSub(subKey);
        setDays(1);
        setPrice(null);
        setError(null);
        setShowModal(true);
    };

    const fetchPrice = async () => {
        setLoadingPrice(true);
        axios.get(apiUrl(`/buy/subscription/price?days=${days}&target=${selectedSub}`))
            .then(response => {
                setPrice(response.data.price);
                setError(null);
            })
            .catch(err => {
                setPrice(null);
                setError(err.response?.data?.message || 'Failed to fetch subscription price');
            })
            .finally(() => setLoadingPrice(false));
    };

    const handleSubscribe = async () => {
        axios.post(apiUrl('/buy/subscription'), {days, target: selectedSub}, {withCredentials: true})
            .then(response => {
                window.location.href = response.data.payment_url;
            })
            .catch(err => {
                console.error(err);
                setError(err.response?.data?.message || 'Subscription request failed.');
                setPrice(null);
            })
    };

    return (
        <Container className="d-flex flex-column">
            <h3>Subscriptions</h3>
            <div className="me-auto">
                {Object.keys(subscriptions).sort().map((key) => {
                    const expiryDate = subscriptions[key];
                    const isActive = expiryDate && new Date(expiryDate) > new Date();

                    return (
                        <Row key={key} className="mb-2">
                            <Col>
                                <Card>
                                    <Card.Body className="d-flex justify-content-between align-items-center">
                                        <span className="me-4">{formatSubscriptionKey(key)}</span>
                                        {isActive ? (
                                            <Badge bg="success">
                                                <LocalDate date={expiryDate}/>
                                            </Badge>
                                        ) : (
                                            <Button variant="dark" size="sm" onClick={() => handleOpenModal(key)}>
                                                Subscribe
                                            </Button>
                                        )}
                                    </Card.Body>
                                </Card>
                            </Col>
                        </Row>
                    );
                })}
            </div>

            <Modal show={showModal} onHide={() => setShowModal(false)} centered>
                <Modal.Header closeButton>
                    <Modal.Title>Subscribe</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    <Form.Group>
                        <Form.Label>Days (1-45)</Form.Label>
                        <Form.Control type="number" min="1" max="45" value={days}
                                      onChange={(e) => setDays(parseInt(e.target.value))}/>
                    </Form.Group>
                    <Button variant="dark" className="mt-2" onClick={fetchPrice} disabled={loadingPrice}>
                        {loadingPrice ? <Spinner size="sm" animation="border"/> : 'Get Price'}
                    </Button>
                    {price && <Alert variant="success" className="mt-2">Price: ${price}</Alert>}
                    {error && <Alert variant="danger" className="mt-2">{error}</Alert>}
                </Modal.Body>
                <Modal.Footer>
                    <Button variant="outline-dark" onClick={() => setShowModal(false)}>Close</Button>
                    <Button variant="dark" onClick={handleSubscribe} disabled={!price}>Subscribe</Button>
                </Modal.Footer>
            </Modal>
        </Container>
    );
}

export default Subscriptions;