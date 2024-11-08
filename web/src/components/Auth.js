import React, { useState } from 'react';
import { Form, Button, Container, Card, Alert } from 'react-bootstrap';
import { auth, signInWithEmailAndPassword, createUserWithEmailAndPassword, getFirebaseErrorMessage } from '../firebase';

function Auth({ onLogin }) {
    const [formData, setFormData] = useState({ email: '', password: '', confirmPassword: '' });
    const [error, setError] = useState('');
    const [isSignUp, setIsSignUp] = useState(false);

    const handleChange = (e) => setFormData({ ...formData, [e.target.name]: e.target.value });
    const toggleMode = () => { setIsSignUp(!isSignUp); setError(''); };

    const handleEmailAuth = async () => {
        const { email, password, confirmPassword } = formData;
        setError('');

        if (!email.trim()) return setError('Please enter your email');
        if (!password.trim()) return setError('Please enter your password');
        if (isSignUp && (password.length < 6 || password !== confirmPassword))
            return setError(password.length < 6 ? 'Password must be at least 6 characters' : 'Passwords do not match');

        try {
            const userCredential = isSignUp
                ? await createUserWithEmailAndPassword(auth, email, password)
                : await signInWithEmailAndPassword(auth, email, password);
            onLogin(await userCredential.user.getIdToken());
        } catch (error) {
            setError(getFirebaseErrorMessage(error));
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
                                name="email"
                                autocomplete="off"
                                value={formData.email}
                                onChange={handleChange}
                                isInvalid={isSignUp && !formData.email.trim()}
                            />
                        </Form.Group>

                        <Form.Group controlId="formPassword" className="mt-3">
                            <Form.Label>Password</Form.Label>
                            <Form.Control
                                type="password"
                                placeholder="Password"
                                name="password"
                                autocomplete="new-password"
                                value={formData.password}
                                onChange={handleChange}
                                isInvalid={isSignUp && formData.password.length > 0 && formData.password.length < 6}
                            />
                        </Form.Group>

                        {isSignUp && (
                            <Form.Group controlId="formConfirmPassword" className="mt-3">
                                <Form.Label>Confirm Password</Form.Label>
                                <Form.Control
                                    type="password"
                                    placeholder="Confirm Password"
                                    name="confirmPassword"
                                    value={formData.confirmPassword}
                                    onChange={handleChange}
                                    isInvalid={formData.password !== formData.confirmPassword}
                                />
                            </Form.Group>
                        )}

                        <Button
                            variant="primary"
                            className="w-100 mt-4"
                            onClick={handleEmailAuth}
                            disabled={isSignUp && formData.password !== formData.confirmPassword}
                        >
                            {isSignUp ? 'Sign Up' : 'Login'}
                        </Button>
                    </Form>

                    <div className="text-center mt-3">
                        <small>
                            {isSignUp ? 'Already have an account?' : "Don't have an account?"}{' '}
                            <Button variant="link" onClick={toggleMode}>
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
