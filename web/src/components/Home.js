import React, {useEffect, useState} from 'react';
import { useNavigate } from 'react-router-dom';
import axios from 'axios';
import {Button, Container, Form, Modal} from 'react-bootstrap';
import MetaMaskButton from './MetaMaskButton';
import InvoiceTable from './InvoiceTable';
import {apiUrl, getBlockchainInfo, NETWORKS} from "../utils";

function Home({isLoggedIn}) {
    const navigate = useNavigate();
    const [showModal, setShowModal] = useState(false);
    const [newInvoice, setNewInvoice] = useState({amount: '', seller: '', networks: []});
    const [creating, setCreating] = useState(false);
    const [networks, setNetworks] = useState([]);

    useEffect(() => {
        const fetchBlockchainInfo = async () => {
            try {
                const response = await getBlockchainInfo();
                const {networks} = response.data;
                const toSetNetworks = networks.map((item) => item.id);

                setNetworks(toSetNetworks);
            } catch (err) {
                console.error('Failed to fetch blockchain info');
            }
        };

        fetchBlockchainInfo();
    }, [])

    const handleCreateInvoice = () => {
        setCreating(true);
        axios
            .post(apiUrl('/payment/invoice'), {
                amount: newInvoice.amount,
                seller: newInvoice.seller,
                networks: newInvoice.networks,
            }, {withCredentials: true})
            .then((response) => {
                setCreating(false);
                setShowModal(false);
                setNewInvoice({amount: '', seller: '', networks: []});
                const newInvoiceId = response.data.id;
                navigate(`/invoices/${newInvoiceId}`)
            })
            .catch((err) => {
                console.log("Failed to create invoice", err);
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

    return (
        <Container className="mt-5">
            <h2>Invoice List</h2>

            <Button variant="primary" onClick={() => setShowModal(true)} className="mb-3">
                Create Invoice
            </Button>

            {/* Invoice Table */}
            <InvoiceTable isLoggedIn={isLoggedIn}/>

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
                                <MetaMaskButton onPress={handleUseMetaMaskAddress}></MetaMaskButton>
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
                                                            ? prev.networks.filter(id => id !== selectedId)
                                                            : [...prev.networks, selectedId]
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
                        disabled={
                            creating || !newInvoice.amount || !newInvoice.seller || newInvoice.networks.length === 0
                        }
                    >
                        {creating ? 'Creating...' : 'Create Invoice'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </Container>
    );
}

export default Home;
