import React, {useEffect, useState} from 'react';
import {
    Container,
    Row,
    Col,
    Card,
    Badge,
    Button,
    Modal,
    Form,
    Alert,
    Spinner,
    OverlayTrigger,
    Tooltip
} from 'react-bootstrap';
import {FaQuestionCircle} from 'react-icons/fa';
import axios from 'axios';
import LocalDate from "../common/LocalDate";
import {apiUrl, getSubscriptionInfo} from '../../utils';

const DEFAULT_DAYS = 1;
const MIN_DAYS = 1;
const MAX_DAYS = 70;

function Subscriptions({subscriptions}) {
    const [showModal, setShowModal] = useState(false);
    const [selectedSub, setSelectedSub] = useState(null);

    const formatSubscriptionKey = (key) => key.replace(/_/g, ' ').replace(/\b\w/g, (char) => char.toUpperCase());

    const handleOpenModal = (subKey) => {
        setSelectedSub(subKey);
        setShowModal(true);
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
                                        <div className="d-flex align-items-center">
                                            <span className="me-2">{formatSubscriptionKey(key)}</span>
                                        </div>
                                        <OverlayTrigger
                                            placement="bottom"
                                            variant="light"
                                            overlay={<Tooltip>{getSubscriptionInfo(key)}</Tooltip>}
                                        >
                                            <Button className="ms-auto me-3" variant="light" size="sm">
                                                <FaQuestionCircle
                                                    className="text-secondary"
                                                    color="blue"
                                                    size="2em"
                                                    style={{ cursor: 'pointer' }}
                                                />
                                            </Button>
                                        </OverlayTrigger>
                                        {isActive ? (
                                            <Badge bg="success">
                                                <LocalDate date={expiryDate}/>
                                            </Badge>
                                        ) : (
                                            <Button className="mx-2" variant="dark" size="sm" onClick={() => handleOpenModal(key)}>
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

            <SubscribeWindow
                startDays={DEFAULT_DAYS}
                selectedSub={selectedSub}
                showModal={showModal}
                closeModal={() => setShowModal(false)}
            />
        </Container>
    );
}

function SubscribeWindow({startDays, selectedSub, showModal, closeModal}) {
    const target = selectedSub;
    const [loadingPrice, setLoadingPrice] = useState(false);
    const [days, setDays] = useState(startDays);
    const [price, setPrice] = useState(null);
    const [error, setError] = useState(null);

    useEffect(() => {
        if (!days || !target) {
            return;
        }

        setLoadingPrice(true);
        fetchPrice(days, target)
            .then(response => {
                setPrice(response.data.price);
                setError(null);
            })
            .catch(err => {
                setPrice(null);
                setError(err.response?.data?.message || 'Failed to fetch subscription price');
            })
            .finally(() => setLoadingPrice(false));
    }, [days, target])

    const closeWithReset = () => {
        closeModal();
        setDays(DEFAULT_DAYS);
        setPrice(null);
        setError(null);
    }

    const handleSubscribe = async () => {
        axios.post(apiUrl('/buy/subscription'), {days: parseInt(days), target}, {withCredentials: true})
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
        <Modal show={showModal} onHide={closeWithReset} animation={false} centered>
            <Modal.Header closeButton>
                <Modal.Title>Subscribe</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <Form.Group>
                    <Form.Label>Days (1-70)</Form.Label>
                    <Form.Control
                        type="number"
                        min={MIN_DAYS}
                        max={MAX_DAYS}
                        value={days}
                        onChange={(e) => setDays(e.target.value)}
                        isInvalid={!days}
                    />
                </Form.Group>
                {price && <Alert variant="success" className="mt-2">Price: ${loadingPrice ? (
                    <Spinner size="sm" animation="border"/>
                ) : price}</Alert>}
                {error && <Alert variant="danger" className="mt-2">{error}</Alert>}
            </Modal.Body>
            <Modal.Footer>
                <Button variant="outline-dark" onClick={closeWithReset}>Close</Button>
                <Button variant="dark" onClick={handleSubscribe}>Subscribe</Button>
            </Modal.Footer>
        </Modal>
    )
}

const fetchPrice = async (days, target) => {
    return await axios.get(
        apiUrl(`/buy/subscription/price?days=${days}&target=${target}`)
    );
};

export default Subscriptions;