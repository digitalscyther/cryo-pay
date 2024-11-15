import React from 'react';
import { Container, Alert } from 'react-bootstrap';

function NotFound() {
    return (
        <Container className="mt-5 text-center">
            <Alert variant="warning">
                <h4>404 - Page Not Found</h4>
                <p>The requested page does not exist.</p>
            </Alert>
        </Container>
    );
}

export default NotFound;
