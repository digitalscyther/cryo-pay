import {Badge, Button, Collapse, Form, Modal, OverlayTrigger, Tooltip} from "react-bootstrap";
import MetaMaskButton from "./MetaMaskButton";
import {apiUrl, exampleEthereumAddressHere, getBlockchainInfo, NETWORKS, sortNetworkItems} from "../../utils";
import React, {useEffect, useState} from "react";
import {useNavigate} from "react-router-dom";
import axios from "axios";
import NetworkIcon from "../NetworkIcon";

const ExternalIdInfo = ({id, children, title}) => (
    <OverlayTrigger overlay={<Tooltip id={id}>{title}</Tooltip>}>
        {children}
    </OverlayTrigger>
);

function CreateInvoice() {
    const navigate = useNavigate();
    const [showModal, setShowModal] = useState(false);
    const [newInvoice, setNewInvoice] = useState({amount: '', seller: '', networks: [], external_id: ''});
    const [creating, setCreating] = useState(false);
    const [networks, setNetworks] = useState([]);
    const [error, setError] = useState(null);
    const [validationError, setValidationError] = useState(null);
    const [showExternalId, setShowExternalId] = useState(false);

    useEffect(() => {
        const fetchBlockchainInfo = async () => {
            try {
                const response = await getBlockchainInfo();
                const {networks} = response.data;
                const toSetNetworks = networks.sort(sortNetworkItems).map((item) => item.id);

                setNetworks(toSetNetworks);
            } catch (err) {
                console.error('Failed to fetch blockchain info');
            }
        };

        fetchBlockchainInfo();
    }, []);

    // Validation helper functions
    const isValidEthereumAddress = (address) => /^0x[a-fA-F0-9]{40}$/.test(address);
    const isValidAmount = (amount) => parseFloat(amount) > 0;

    const handleCreateInvoice = () => {
        setValidationError(null);
        setError(null);

        // Validation checks
        if (!isValidEthereumAddress(newInvoice.seller)) {
            setValidationError(
                <>
                    Seller address must be a valid Ethereum address like (e.g., <code>{exampleEthereumAddressHere}</code>).
                </>
            );
            return;
        }

        if (!isValidAmount(newInvoice.amount)) {
            setValidationError("Amount must be greater than 0.");
            return;
        }

        if (newInvoice.networks.length === 0) {
            setValidationError("At least one network must be selected.");
            return;
        }

        if (newInvoice.external_id.length > 255) {
            setValidationError("External ID must be less than 255 characters.");
            return;
        }

        // Prepare data for API call
        const data = {
            amount: newInvoice.amount,
            seller: newInvoice.seller,
            networks: newInvoice.networks,
        };

        let external_id = (newInvoice.external_id || "").trim() || null;
        if (external_id) {
            data.external_id = newInvoice.external_id;
        }

        // If validations pass, proceed with API call
        setCreating(true);
        axios
            .post(apiUrl('/payment/invoice'), data, {withCredentials: true})
            .then((response) => {
                setCreating(false);
                setShowModal(false);
                setNewInvoice({amount: '', seller: '', networks: [], external_id: ''});
                setShowExternalId(false);
                const newInvoiceId = response.data.id;
                navigate(`/invoices/${newInvoiceId}`);
            })
            .catch((err) => {
                setCreating(false);
                if (err.response && err.response.status === 429) {
                    setError("You have reached the limit of creating invoices for today. Please try again tomorrow.");
                } else {
                    console.error("Failed to create invoice", err);
                }
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

    let externalIdInfoText = (
        "This ID will be visible to the payer. You can use it as a name or for any other purpose."
    );

    return (
        <>
            <Button variant="primary" onClick={() => setShowModal(true)} className="mb-3">
                Create Invoice
            </Button>

            <Modal show={showModal} onHide={() => setShowModal(false)}>
                <Modal.Header closeButton>
                    <Modal.Title>Create Invoice</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    {
                        error && <div className="alert alert-danger" role="alert">
                            <>{error}</>
                            <div className="d-flex">
                                <small className="mt-2 ms-auto" style={{fontSize: "0.8em"}}>
                                    More info about limits <a href="/docs#limits">here</a>
                                </small>
                            </div>
                        </div>
                    }
                    {
                        validationError && <div className="alert alert-warning" role="alert">
                            {validationError}
                        </div>
                    }
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
                                            id={`invoice-network-${network.id}`}
                                            type="checkbox"
                                            label={(
                                                <div className="my-2 mx-4">
                                                    <NetworkIcon size={30} networkName={network.name} cursor={'pointer'} />
                                                </div>
                                            )}
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
                                            className="d-flex align-items-center m-0"
                                        />
                                    ))}
                            </div>
                        </Form.Group>
                        <Form.Group controlId="formExternalId" className="mt-3">
                            <Button
                                variant="outline-primary"
                                onClick={() => setShowExternalId(!showExternalId)}
                                aria-controls="external-id-collapse"
                                aria-expanded={showExternalId}
                            >
                                Add External ID
                            </Button>
                            <Collapse in={showExternalId}>
                                <div id="external-id-collapse">
                                    <Form.Label>
                                        External ID (Optional){" "}<ExternalIdInfo
                                        title={externalIdInfoText}
                                        id="external-id-link"
                                    ><Badge pill bg="secondary"> ? </Badge></ExternalIdInfo>
                                    </Form.Label>
                                    <Form.Control
                                        type="text"
                                        placeholder="Enter external ID"
                                        value={newInvoice.external_id}
                                        onChange={(e) => setNewInvoice({
                                            ...newInvoice,
                                            external_id: e.target.value
                                        })}
                                    />
                                </div>
                            </Collapse>
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
        </>
    );
}

export default CreateInvoice;
