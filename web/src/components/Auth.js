import React, { useState } from 'react';
import { Form, Button, Container, Card, Alert } from 'react-bootstrap';
import { auth, signInWithEmailAndPassword, createUserWithEmailAndPassword, googleProvider, appleProvider, signInWithPopup } from '../firebase';

const APPLE_LOGIN = false;

function Auth({ onLogin }) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const [isSignUp, setIsSignUp] = useState(false);

    const handleEmailAuth = async () => {
        try {
            let userCredential;
            if (isSignUp) {
                userCredential = await createUserWithEmailAndPassword(auth, email, password);
            } else {
                userCredential = await signInWithEmailAndPassword(auth, email, password);
            }
            const idToken = await userCredential.user.getIdToken();
            onLogin(idToken);
        } catch (error) {
            setError(error.message);
        }
    };

    const handleGoogleSignIn = async () => {
        try {
            const result = await signInWithPopup(auth, googleProvider);
            const idToken = await result.user.getIdToken();
            onLogin(idToken);
        } catch (error) {
            setError(error.message);
        }
    };

    const handleAppleSignIn = async () => {
        try {
            const result = await signInWithPopup(auth, appleProvider);
            const idToken = await result.user.getIdToken();
            onLogin(idToken);
        } catch (error) {
            setError(error.message);
        }
    };

    return (
        <Container className="d-flex justify-content-center align-items-center" style={{ minHeight: '100vh' }}>
            <Card style={{ width: '100%', maxWidth: '400px' }}>
                <Card.Body>
                    <h2 className="text-center mb-4">{isSignUp ? 'Sign Up' : 'Login'}</h2>

                    {error && <Alert variant="danger">{error}</Alert>}

                    <Form>
                        <Form.Group controlId="formEmail">
                            <Form.Label>Email address</Form.Label>
                            <Form.Control
                                type="email"
                                placeholder="Enter email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                            />
                        </Form.Group>

                        <Form.Group controlId="formPassword" className="mt-3">
                            <Form.Label>Password</Form.Label>
                            <Form.Control
                                type="password"
                                placeholder="Password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                            />
                        </Form.Group>

                        <Button variant="primary" className="w-100 mt-4" onClick={handleEmailAuth}>
                            {isSignUp ? 'Sign Up' : 'Login'}
                        </Button>

                        <hr className="my-4" />

                        <Button variant="outline-primary" className="w-100 mb-2" onClick={handleGoogleSignIn}>
                            Sign in with Google
                        </Button>

                        {APPLE_LOGIN && (
                            <Button variant="outline-secondary" className="w-100 mb-2" onClick={handleAppleSignIn}>
                                Sign in with Apple
                            </Button>
                        )}
                    </Form>

                    <div className="text-center mt-3">
                        <small>
                            {isSignUp ? 'Already have an account?' : "Don't have an account?"}{' '}
                            <Button variant="link" onClick={() => setIsSignUp(!isSignUp)}>
                                {isSignUp ? 'Login' : 'Sign Up'}
                            </Button>
                        </small>
                    </div>
                </Card.Body>
            </Card>
        </Container>
    );
}

export default Auth;
