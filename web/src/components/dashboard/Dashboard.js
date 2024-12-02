import React from 'react';
import {Container} from 'react-bootstrap';
import InvoiceList from "./InvoiceList";
import CreateInvoice from "./CreateInvoice";

function Dashboard({isLoggedIn}) {
    return (
        <Container className="mt-5">
            <h2>Invoice List</h2>

            {/* Creating New Invoice */}
            <CreateInvoice />

            {/* Invoice Table */}
            <InvoiceList isLoggedIn={isLoggedIn}/>

        </Container>
    );
}

export default Dashboard;
