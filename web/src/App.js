import React, { useEffect, useState} from 'react';
import {Route, Routes, useNavigate} from 'react-router-dom';
import {Container, Navbar, Nav} from 'react-bootstrap';
import axios from "axios";
import { jwtDecode } from "jwt-decode";
import Cookies from "js-cookie";
import Home from './components/Home';
import About from './components/About';
import Contact from './components/Contact';
import Invoice from './components/Invoice';
import Auth from './components/Auth';
import {apiUrl} from "./utils";

function App() {
    const navigate = useNavigate();
    const [isLoggedIn, setIsLoggedIn] = useState(false);

    useEffect(() => {
        const checkAuth = () => {
            const token = Cookies.get("jwt");
            if (token) {
                try {
                    const decoded = jwtDecode(token);
                    const currentTime = Date.now() / 1000;
                    if (decoded.exp > currentTime) {
                        setIsLoggedIn(true);
                    } else {
                        setIsLoggedIn(false);
                        Cookies.remove("jwt");
                    }
                } catch (error) {
                    console.error("Invalid JWT token:", error);
                    setIsLoggedIn(false);
                }
            } else {
                setIsLoggedIn(false);
            }
        };

        checkAuth();
    }, []);

    const handleLogin = (token) => {
        axios
            .post(apiUrl('/auth/login'), {token}, {withCredentials: true})
            .then(() => {
                setIsLoggedIn(true);
            })
            .catch((err) => {
                console.log("Failed to login", err);
            });
        navigate('/');
    };

    const handleLogout = () => {
        axios
            .post(apiUrl('/auth/logout'), {}, {withCredentials: true})
            .then((response) => {
                setIsLoggedIn(false);
            })
            .catch((err) => {
                console.log("Failed to logout", err);
            });
        navigate('/');
    };

    return (
        <>
            <Navbar bg="dark" variant="dark" expand="lg">
                <Container>
                    <Navbar.Brand href="/">MyApp</Navbar.Brand>
                    <Navbar.Toggle aria-controls="basic-navbar-nav"/>
                    <Navbar.Collapse id="basic-navbar-nav">
                        <Nav className="d-flex w-100">
                            <Nav.Link href="/">Home</Nav.Link>
                            <Nav.Link href="/about">About</Nav.Link>
                            <Nav.Link href="/contact">Contact</Nav.Link>
                            {!isLoggedIn ? (
                                <Nav.Link href="/login">Login</Nav.Link>
                            ) : (
                                <Nav.Link className="ms-auto" onClick={handleLogout}>Logout</Nav.Link>
                            )}
                        </Nav>
                    </Navbar.Collapse>
                </Container>
            </Navbar>

            <Container className="mt-3">
                <Routes>
                    <Route path="/" element={<Home isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/invoices/:invoice_id" element={<Invoice/>}/>
                    <Route path="/about" element={<About/>}/>
                    <Route path="/contact" element={<Contact/>}/>
                    <Route path="/login" element={<Auth onLogin={handleLogin}/>}/>
                </Routes>
            </Container>
        </>
    );
}

export default App;
