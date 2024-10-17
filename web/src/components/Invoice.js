import React, {useEffect, useState} from 'react';
import {useParams} from 'react-router-dom';
import axios from 'axios';
import Web3 from 'web3';
import {Container, Button, Spinner, Alert} from 'react-bootstrap';
import {apiUrl, getBlockchainInfo, getNetwork} from "../utils";

function Invoice() {
    const {invoice_id} = useParams();
    const [invoice, setInvoice] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [processing, setProcessing] = useState(false);
    const [erc20Abi, setErc20Abi] = useState(null);
    const [contractAbi, setContractAbi] = useState(null);
    const [networks, setNetworks] = useState(null);

    useEffect(() => {
        const fetchInvoice = async () => {
            try {
                const response = await axios.get(apiUrl(`/payment/invoice/${invoice_id}`));
                setInvoice(response.data);
            } catch (err) {
                setError('Failed to fetch invoice data');
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
    }, [invoice_id]);

    const handlePayment = async () => {
        const web3 = new Web3(window.ethereum);

        const getNetworkId = () => web3.eth.net.getId();

        const networkId = Number(await getNetworkId());
        const network = networks[networkId];
        if (!invoice.networks.includes(networkId) || !network) {
            return setError('Please switch to the correct network');
        }

        const isValidState = () => web3 && account && invoice && erc20Contract && invoiceContract;
        const handleApproval = (amount) => () => erc20Contract.methods.approve(invoiceContract._address, amount).send({from: account});
        const handlePaymentTransaction = (amount) => () => invoiceContract.methods.payInvoice(invoice.seller, invoice_id, amount).send({from: account});

        const erc20Contract = new web3.eth.Contract(erc20Abi, network.addresses.erc20);
        const invoiceContract = new web3.eth.Contract(contractAbi, network.addresses.contract);
        const accounts = await window.ethereum.request({method: 'eth_requestAccounts'});
        const account = accounts[0];

        setError(null);
        if (!isValidState()) return;

        const processPayment = async (amount) =>
            handleApproval(amount)()
                .then(() => console.log('Approval successful'))
                .then(handlePaymentTransaction(amount))
                .then(() => console.log('Payment successful'));

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

    if (loading) {
        return (
            <Container className="mt-5 text-center">
                <Spinner animation="border" variant="primary"/>
            </Container>
        );
    }

    return (
        <Container className="mt-5">
            <h2>Pay Invoice</h2>
            <p><strong>Invoice ID:</strong> {invoice.id}</p>
            <p><strong>Amount:</strong> {parseFloat(invoice.amount).toFixed(2)} MTK</p>
            <div><strong>Networks:</strong><br/>
                {invoice.networks.length > 0 ? (
                    <ul style={{listStyleType: "none", paddingLeft: "20px"}}>
                        {invoice.networks.map((n) => (
                            <li key={n}>– {getNetwork(n).name}</li>
                        ))}
                    </ul>
                ) : (
                    <span>&nbsp;&nbsp;&nbsp;&nbsp;No networks available</span>
                )}
            </div>
            <p><strong>Seller:</strong> {invoice.seller}</p>
            <p><strong>Created At:</strong> {new Date(invoice.created_at).toLocaleString()}</p>

            {invoice.paid_at ? (
                <>
                    <p><strong>Paid At:</strong> {new Date(invoice.paid_at).toLocaleString()}</p>
                    <Alert variant="success">This invoice has already been paid.</Alert>
                </>
            ) : (
                <>
                    {processing ? (
                        <Button variant="primary" disabled>
                            Processing Payment...
                        </Button>
                    ) : (
                        <Button variant="primary" onClick={handlePayment}>
                            Pay with MetaMask
                        </Button>
                    )}
                </>
            )}

            {error && <Alert className="my-3" variant="danger">{error}</Alert>}

        </Container>
    );
}

export default Invoice;
