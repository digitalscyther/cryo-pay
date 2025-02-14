import React, { useEffect, useState } from 'react';
import { Table, Spinner, Alert } from 'react-bootstrap';
import axios from 'axios';
import { apiUrl } from '../../utils';
import AmountDisplay from "../common/AmountDisplay";
import LocalDate from "../common/LocalDate";

function DonationList() {
    const [donations, setDonations] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        axios.get(apiUrl('/buy/donation'), { withCredentials: true })
            .then(response => setDonations(response.data))
            .catch(err => {
                setError('Failed to load donations. Please try again.');
                console.error('Error fetching donations:', err);
            })
            .finally(() => setLoading(false));
    }, []);

    return (
        <>
            {loading && <Spinner animation="border" className="d-block mx-auto my-4" />}
            {error && <Alert variant="danger" className="text-center">{error}</Alert>}
            {!loading && !error && (
                <Table striped bordered hover responsive className="mt-3">
                    <thead className="bg-dark text-light">
                    <tr>
                        <th>#</th>
                        <th>ID</th>
                        <th>Donor</th>
                        <th>Target</th>
                        <th>Amount</th>
                        <th>Paid At</th>
                    </tr>
                    </thead>
                    <tbody>
                        {donations.length > 0 ? donations.map((donation, index) => (
                            <tr key={donation.id}>
                                <td>{index + 1}</td>
                                <td>...{donation.id.slice(-5)}</td>
                                <td>{donation.donor || "Anonymus"}</td>
                                <td>{donation.target || "For Any Purpose"}</td>
                                <td className="fw-bold"><AmountDisplay amount={donation.amount} /></td>
                                <td><LocalDate date={donation.paid_at}/></td>
                            </tr>
                        )) : (
                            <tr>
                            <td colSpan="6" className="text-center text-muted py-3">No donations yet.</td>
                            </tr>
                        )}
                    </tbody>
                </Table>
            )}
        </>
    );
}

export default DonationList;
