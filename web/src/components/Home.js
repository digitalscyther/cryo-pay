import React, {useEffect, useState} from 'react';
import axios from 'axios';
import {Table, Container, Alert, Spinner, Button, Modal, Form} from 'react-bootstrap';
import {apiUrl, getBlockchainInfo, NETWORKS} from "../utils";


const PAGE_SIZE = 10;

function Home() {
    const [invoices, setInvoices] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [showModal, setShowModal] = useState(false);
    const [newInvoice, setNewInvoice] = useState({amount: '', seller: '', networks: []});
    const [creating, setCreating] = useState(false);
    const [offset, setOffset] = useState(0);
    const [hasMore, setHasMore] = useState(false);
    const limit = PAGE_SIZE;
    const [networks, setNetworks] = useState([]);

    useEffect(() => {
        setLoading(true);

        const fetchBlockchainInfo = async () => {
            try {
                const response = await getBlockchainInfo();
                const {networks} = response.data;
                const toSetNetworks = networks.map((item) => item.id);

                setNetworks(toSetNetworks);
            } catch (err) {
                setError('Failed to fetch blockchain info');
            }
        };

        fetchBlockchainInfo();

        axios
            .get(apiUrl(`/payment/invoice?limit=${limit}&offset=${offset}`))
            .then((response) => {
                setInvoices(response.data);
                setHasMore(response.data.length === limit);
                setLoading(false);
            })
            .catch((err) => {
                console.log(`Failed get invoices: ${err}`);
                setError('Failed to fetch invoices');
                setLoading(false);
            });
    }, [limit, offset]);

    const handleCreateInvoice = () => {
        setCreating(true);
        axios
            .post(apiUrl('/payment/invoice'), {
                amount: newInvoice.amount,
                seller: newInvoice.seller,
                networks: newInvoice.networks,
            })
            .then((response) => {
                setInvoices([response.data, ...invoices]);
                setOffset(0);
                setCreating(false);
                setShowModal(false);
                setNewInvoice({amount: '', seller: '', networks: []});
            })
            .catch((err) => {
                console.log("Failed to create invoice", err);
                setError('Failed to create invoice');
                setCreating(false);
            });
    };

    const handleSellerChange = (e) => {
        const textarea = e.target;
        textarea.style.height = 'auto';
        textarea.style.height = textarea.scrollHeight + 'px';
        setNewInvoice({...newInvoice, seller: textarea.value});
    };

    const handleUseMetaMaskAddress = async () => {
        if (window.ethereum) {
            try {
                const accounts = await window.ethereum.request({method: 'eth_requestAccounts'});
                const address = accounts[0];
                setNewInvoice({...newInvoice, seller: address});
            } catch (error) {
                console.error('Error fetching MetaMask address:', error);
            }
        } else {
            console.error('MetaMask is not installed.');
        }
    };

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary"/>
            </Container>
        );
    }

    if (error) {
        return (
            <Container className="mt-5">
                <Alert variant="danger">{error}</Alert>
            </Container>
        );
    }

    return (
        <Container className="mt-5">
            <h2>Invoice List</h2>

            {/* Create Invoice Button */}
            <Button variant="primary" onClick={() => setShowModal(true)} className="mb-3">
                Create Invoice
            </Button>

            <div className="d-flex justify-content-end mt-3 mb-1 me-3">
                <Button
                    variant="primary"
                    disabled={offset === 0}
                    onClick={() => setOffset(offset - limit)}
                    className="me-2"
                >
                    &laquo; {/* Unicode for left double angle quotation mark */}
                </Button>

                <Button
                    variant="primary"
                    disabled={!hasMore}
                    onClick={() => setOffset(offset + limit)}
                >
                    &raquo; {/* Unicode for right double angle quotation mark */}
                </Button>
            </div>

            {invoices.length === 0 ? (
                <Alert variant="info">No invoices found</Alert>
            ) : (
                <Table striped bordered hover responsive>
                    <thead>
                    <tr>
                        <th>ID</th>
                        <th>Amount</th>
                        {/*<th>Seller</th>*/}
                        {/*<th>Buyer</th>*/}
                        <th>Created At</th>
                        <th>Paid At</th>
                    </tr>
                    </thead>
                    <tbody>
                    {invoices.map((invoice) => (
                        <tr key={invoice.id}>
                            <td>
                                <Button
                                    variant="link"
                                    onClick={() => window.open(`/invoices/${invoice.id}`, '_blank')}
                                >
                                    {invoice.id}
                                </Button>
                            </td>
                            <td>{parseFloat(invoice.amount).toFixed(2)}</td>
                            {/*<td>{invoice.seller}</td>*/}
                            {/*<td>{invoice.buyer || 'N/A'}</td>*/}
                            <td>{new Date(invoice.created_at).toLocaleString()}</td>
                            <td>{invoice.paid_at ? new Date(invoice.paid_at).toLocaleString() : 'Unpaid'}</td>
                        </tr>
                    ))}
                    </tbody>
                </Table>
            )}

            {/* Modal for Creating New Invoice */}
            <Modal show={showModal} onHide={() => setShowModal(false)}>
                <Modal.Header closeButton>
                    <Modal.Title>Create Invoice</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    <Form>
                        <Form.Group controlId="formAmount">
                            <Form.Label>Amount</Form.Label>
                            <Form.Control
                                type="text"
                                placeholder="Enter amount"
                                value={newInvoice.amount}
                                onChange={(e) => setNewInvoice({...newInvoice, amount: e.target.value})}
                            />
                        </Form.Group>

                        <Form.Group controlId="formSeller" className="mt-3">
                            <Form.Label>Seller</Form.Label>
                            <div className="d-flex">
                                <Form.Control
                                    as="textarea"
                                    rows={1}
                                    placeholder="Enter seller address"
                                    value={newInvoice.seller}
                                    onChange={handleSellerChange}
                                    style={{resize: 'none', overflow: 'hidden'}}
                                />
                                <div><Button
                                    variant="outline-primary"
                                    className="ms-2"
                                    onClick={handleUseMetaMaskAddress}
                                >
                                    Use MetaMask
                                </Button></div>
                            </div>
                        </Form.Group>

                        <Form.Group controlId="formNetworkId" className="mt-3">
                            <Form.Label>Select Network(s)</Form.Label>
                            <div>
                                {Object.values(NETWORKS)
                                    .sort((a, b) => a.order - b.order)
                                    .map((network) => (
                                        <Form.Check
                                            key={network.id}
                                            type="checkbox"
                                            label={network.name}
                                            id={`invoice-network-${network.id}`}
                                            value={network.id}
                                            checked={newInvoice.networks.includes(network.id)}
                                            disabled={!networks.includes(network.id)}
                                            onChange={(e) => {
                                                const selectedId = parseInt(e.target.value);
                                                setNewInvoice((prev) => {
                                                    const isSelected = prev.networks.includes(selectedId);
                                                    return {
                                                        ...prev,
                                                        networks: isSelected
                                                            ? prev.networks.filter(id => id !== selectedId) // Deselect
                                                            : [...prev.networks, selectedId] // Select
                                                    };
                                                });
                                            }}
                                        />
                                    ))}
                            </div>
                        </Form.Group>
                    </Form>
                </Modal.Body>
                <Modal.Footer>
                    <Button variant="secondary" onClick={() => setShowModal(false)}>
                        Close
                    </Button>
                    <Button
                        variant="primary"
                        onClick={handleCreateInvoice}
                        disabled={creating || !newInvoice.amount || !newInvoice.seller || newInvoice.networks.length === 0}
                    >
                        {creating ? 'Creating...' : 'Create Invoice'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </Container>
    );
}

export default Home;
