import {Alert, Button, Col, Row} from "react-bootstrap";
import React, {useState} from "react";
import Web3 from "web3";
import axios from "axios";
import {apiUrl} from "../../utils";
import BN from "bn.js";
import {useNavigate} from "react-router-dom";

function Controller({ invoice, own, erc20Abi, contractAbi, networks, waitPaymentSuccessful }) {
    const navigate = useNavigate();
    const [processingPayment, setPaymentProcessing] = useState(false);
    const [paymentSuccessful, setPaymentSuccessful] = useState(false);
    const [processingDelete, setDeleteProcessing] = useState(false);
    const [error, setError] = useState(null);
    const isMetaMask = !!(window.ethereum || {}).isMetaMask;

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
                .payInvoice(invoice.seller, invoice.id, amount)
                .estimateGas({from: account});

            await invoiceContract.methods
                .payInvoice(invoice.seller, invoice.id, amount)
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
            setPaymentProcessing(true);
            const amount = invoice.amount * (10 ** 6);

            await processPayment(amount);
            setPaymentSuccessful(true);
            waitPaymentSuccessful();
        } catch (error) {
            console.error('Payment failed', error);
            setError('Payment failed, please try again');
        } finally {
            setPaymentProcessing(false);
        }
    };

    const handleDelete = async () => {
        try {
            setDeleteProcessing(true);
            await axios.delete(
                apiUrl(`/payment/invoice/${invoice.id}`),
                {withCredentials: true}
            );
            alert('Invoice deleted successfully.');
            navigate('/dashboard');
        } catch (err) {
            setError('Failed to delete the invoice, please try again.');
        } finally {
            setDeleteProcessing(false);
        }
    };

    return (
        <>
            {error && (
                <Alert variant="danger" className="text-center mt-4">
                    {error}
                </Alert>
            )}
            <Row className="align-items-center">
                <Col className="text-start">
                    <Button
                        variant={!isMetaMask ? "danger" : paymentSuccessful ? "success" : "primary"}
                        onClick={handlePayment}
                        disabled={processingPayment || paymentSuccessful || !isMetaMask}
                    >
                        {!isMetaMask
                            ? 'Need install MetaMask'
                            : paymentSuccessful
                            ? 'Checking...'
                            : processingPayment
                            ? 'Processing Payment...'
                            : 'Pay with MetaMask'}
                    </Button>
                </Col>
                {own && (
                    <Col className="text-end">
                        <Button
                            variant="danger"
                            onClick={handleDelete}
                            disabled={processingDelete}
                        >
                            {processingDelete ? 'Deleting...' : 'Delete Invoice'}
                        </Button>
                    </Col>
                )}
            </Row>
        </>
    )
}

export default Controller;
