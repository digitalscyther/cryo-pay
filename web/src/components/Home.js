import React, { useEffect, useState } from 'react';
import axios from 'axios';
import { Table, Container, Alert, Spinner } from 'react-bootstrap';

function Home() {
  const [invoices, setInvoices] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    // Fetch invoices from backend
    axios
      .get('http://localhost:3000/payment/invoice')
      .then((response) => {
        setInvoices(response.data);
        setLoading(false);
      })
      .catch((err) => {
        setError('Failed to fetch invoices');
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <Container className="mt-5 text-center">
        <Spinner animation="border" variant="primary" />
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
                  <td>{invoice.id}</td>
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
    </Container>
  );
}

export default Home;
