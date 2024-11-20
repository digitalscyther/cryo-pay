import React, {useEffect, useState} from 'react';
import {useParams, useNavigate} from 'react-router-dom';
import axios from 'axios';
import BN from 'bn.js';
import Web3 from 'web3';
import {Alert, Col, Container, Button, ListGroup, Row, Spinner} from 'react-bootstrap';
import AmountDisplay from './AmountDisplay';
import {apiUrl, getBlockchainInfo, getNetwork} from "../utils";

function Invoice() {
    const navigate = useNavigate();
    const {invoice_id} = useParams();
    const [invoice, setInvoice] = useState(null);
    const [own, setOwn] = useState(false);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [processing, setProcessing] = useState(false);
    const [erc20Abi, setErc20Abi] = useState(null);
    const [contractAbi, setContractAbi] = useState(null);
    const [networks, setNetworks] = useState(null);

    useEffect(() => {
        const fetchInvoice = async () => {
            try {
                const response = await axios.get(
                    apiUrl(`/payment/invoice/${invoice_id}`),
                    {withCredentials: true}
                );
                setInvoice(response.data.invoice);
                setOwn(response.data.own);
            } catch (err) {
                if (err.response && err.response.status === 404) {
                    navigate('/not-found');
                } else {
                    setError('Failed to fetch invoice data');
                }
            } finally {
                setLoading(false);
            }
        };

        const fetchBlockchainInfo = async () => {
            try {
                const response = await getBlockchainInfo();
                const {networks, abi} = response.data;

                setErc20Abi(abi.erc20);
                setContractAbi(abi.contract);
                setNetworks(networks.reduce((acc, item) => {
                    acc[item.id] = item;
                    return acc;
                }, {}));
            } catch (err) {
                setError('Failed to fetch blockchain info or connect to MetaMask');
            }
        };

        fetchInvoice();
        fetchBlockchainInfo();
    }, [invoice_id, navigate]);

    const handlePayment = async () => {
        const web3 = new Web3(window.ethereum);

        const getNetworkId = () => web3.eth.net.getId();

        const networkId = Number(await getNetworkId());
        const network = networks[networkId];
        if (!invoice.networks.includes(networkId) || !network) {
            return setError('Please switch to the correct network');
        }

        const isValidState = () => web3 && account && invoice && erc20Contract && invoiceContract;
        const fetchGasFees = async () => {
            const response = await axios.get(apiUrl(`/blockchain/suggested_gas_fees/${networkId}`));
            const {data} = response.data;
            return {
                maxFeePerGas: Web3.utils.toWei(data.medium.suggestedMaxFeePerGas, 'gwei'),
                maxPriorityFeePerGas: Web3.utils.toWei(data.medium.suggestedMaxPriorityFeePerGas, 'gwei'),
            };
        };
        const checkAllowance = async (spender, amount) => {
            const allowance = await erc20Contract.methods.allowance(account, spender).call();
            const allowanceBN = new BN(allowance.toString());
            const amountBN = new BN(amount.toString());
            return allowanceBN.gte(amountBN);
        };
        const handleApproval = async (amount, gasFees) => {
            const gasEstimate = await erc20Contract.methods
                .approve(invoiceContract._address, amount)
                .estimateGas({from: account});

            await erc20Contract.methods
                .approve(invoiceContract._address, amount)
                .send({
                    from: account,
                    gas: gasEstimate,
                    ...(gasFees && {
                        maxFeePerGas: gasFees.maxFeePerGas,
                        maxPriorityFeePerGas: gasFees.maxPriorityFeePerGas,
                    }),
                });
        };
        const handlePaymentTransaction = async (amount, gasFees) => {
            const gasEstimate = await invoiceContract.methods
                .payInvoice(invoice.seller, invoice_id, amount)
                .estimateGas({from: account});

            await invoiceContract.methods
                .payInvoice(invoice.seller, invoice_id, amount)
                .send({
                    from: account,
                    gas: gasEstimate,
                    ...(gasFees && {
                        maxFeePerGas: gasFees.maxFeePerGas,
                        maxPriorityFeePerGas: gasFees.maxPriorityFeePerGas,
                    }),
                });
        };

        const erc20Contract = new web3.eth.Contract(erc20Abi, network.addresses.erc20);
        const invoiceContract = new web3.eth.Contract(contractAbi, network.addresses.contract);
        const accounts = await window.ethereum.request({method: 'eth_requestAccounts'});
        const account = accounts[0];

        setError(null);
        if (!isValidState()) return;

        const processPayment = async (amount) => {
            const gasFees = await fetchGasFees();

            const isAllowanceSufficient = await checkAllowance(invoiceContract._address, amount);
            if (!isAllowanceSufficient) {
                await handleApproval(amount, gasFees);
            }

            await handlePaymentTransaction(amount, gasFees);
        };

        try {
            setProcessing(true);
            const amount = invoice.amount * (10 ** 6);

            await processPayment(amount);
            alert('Payment is under processing. It will be marked as paid once everything is fine.');
        } catch (error) {
            console.error('Payment failed', error);
            setError('Payment failed, please try again');
        } finally {
            setProcessing(false);
        }
    };

    const handleDelete = async () => {
        try {
            setProcessing(true);
            await axios.delete(
                apiUrl(`/payment/invoice/${invoice_id}`),
                {withCredentials: true}
            );
            alert('Invoice deleted successfully.');
            navigate('/');
        } catch (err) {
            setError('Failed to delete the invoice, please try again.');
        } finally {
            setProcessing(false);
        }
    };

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary"/>
            </Container>
        );
    }

    return (
        <Container className="mt-5" style={{maxWidth: '600px'}}>
            <h2 className="mb-4">Pay Invoice</h2>

            <ListGroup variant="flush" className="mb-4">
                <ListGroup.Item>
                    <strong>Invoice ID:</strong> {invoice.id}
                </ListGroup.Item>
                <ListGroup.Item>
                    <strong>Amount:</strong> <AmountDisplay amount={invoice.amount} color={"text-success"} />
                </ListGroup.Item>
                <ListGroup.Item>
                    <strong>Networks:</strong>
                    <div className="mt-2">
                        {invoice.networks.length > 0 ? (
                            <ListGroup variant="flush">
                                {invoice.networks.map((n) => (
                                    <ListGroup.Item key={n} className="border-0 ps-3">
                                        – {getNetwork(n).name}
                                    </ListGroup.Item>
                                ))}
                            </ListGroup>
                        ) : (
                            <span className="ps-3">No networks available</span>
                        )}
                    </div>
                </ListGroup.Item>
                <ListGroup.Item>
                    <strong>Seller:</strong> {invoice.seller}
                </ListGroup.Item>
                <ListGroup.Item>
                    <strong>Created At:</strong>{' '}
                    {new Date(invoice.created_at).toLocaleString()}
                </ListGroup.Item>
                {invoice.paid_at && (
                    <ListGroup.Item>
                        <strong>Paid At:</strong>{' '}
                        {new Date(invoice.paid_at).toLocaleString()}
                    </ListGroup.Item>
                )}
            </ListGroup>

            {invoice.paid_at ? (
                <Alert variant="success" className="text-center">
                    This invoice has already been paid.
                </Alert>
            ) : (
                <Row className="align-items-center">
                    <Col className="text-start">
                        <Button
                            variant="primary"
                            onClick={handlePayment}
                            disabled={processing}
                        >
                            {processing ? 'Processing Payment...' : 'Pay with MetaMask'}
                        </Button>
                    </Col>
                    {own && (
                        <Col className="text-end">
                            <Button
                                variant="danger"
                                onClick={handleDelete}
                                disabled={processing}
                            >
                                {processing ? 'Deleting...' : 'Delete Invoice'}
                            </Button>
                        </Col>
                    )}
                </Row>
            )}

            {error && (
                <Alert variant="danger" className="text-center mt-4">
                    {error}
                </Alert>
            )}
        </Container>
    );
}

export default Invoice;
