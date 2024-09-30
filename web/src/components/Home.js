import React, { useEffect, useState } from 'react';
import axios from 'axios';
import { Table, Container, Alert, Spinner, Button, Modal, Form } from 'react-bootstrap';

function Home() {
  const [invoices, setInvoices] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [showModal, setShowModal] = useState(false);
  const [newInvoice, setNewInvoice] = useState({ amount: '', seller: '' });
  const [creating, setCreating] = useState(false);

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

  const handleCreateInvoice = () => {
    setCreating(true);
    // Make POST request to create a new invoice
    axios
      .post('http://localhost:3000/payment/invoice', {
        amount: newInvoice.amount,
        seller: newInvoice.seller,
      })
      .then((response) => {
        // Prepend the newly created invoice to the list
        setInvoices([response.data, ...invoices]);
        setCreating(false);
        setShowModal(false);
        setNewInvoice({ amount: '', seller: '' });
      })
      .catch((err) => {
        setError('Failed to create invoice');
        setCreating(false);
      });
  };

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

      {/* Create Invoice Button */}
      <Button variant="primary" onClick={() => setShowModal(true)} className="mb-3">
        Create Invoice
      </Button>

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
                onChange={(e) => setNewInvoice({ ...newInvoice, amount: e.target.value })}
              />
            </Form.Group>

            <Form.Group controlId="formSeller" className="mt-3">
              <Form.Label>Seller</Form.Label>
              <Form.Control
                type="text"
                placeholder="Enter seller address"
                value={newInvoice.seller}
                onChange={(e) => setNewInvoice({ ...newInvoice, seller: e.target.value })}
              />
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
            disabled={creating || !newInvoice.amount || !newInvoice.seller}
          >
            {creating ? 'Creating...' : 'Create Invoice'}
          </Button>
        </Modal.Footer>
      </Modal>
    </Container>
  );
}

export default Home;
