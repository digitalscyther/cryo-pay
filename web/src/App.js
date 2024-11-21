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
import Account from './components/Account';
import NotFound from './components/NotFound';
import Dashboard from './components/Dashboard';
import {apiUrl, getProjectName} from "./utils";
import Documentation from "./components/Documentation";

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

    const projectName = getProjectName();

    return (
        <>
            <Navbar bg="dark" variant="dark" expand="lg">
                <Container>
                    <Navbar.Brand href="/">{ projectName }</Navbar.Brand>
                    <Navbar.Toggle aria-controls="basic-navbar-nav"/>
                    <Navbar.Collapse id="basic-navbar-nav">
                        <Nav className="d-flex w-100">
                            <Nav.Link href="/">Home</Nav.Link>
                            <Nav.Link href="/dashboard">Dashboard</Nav.Link>
                            <Nav.Link href="/about">About</Nav.Link>
                            <Nav.Link href="/contact">Contact</Nav.Link>
                            {!isLoggedIn ? (
                                <Nav.Link href="/login">Login</Nav.Link>
                            ) : (
                                <div className="ms-auto d-flex">
                                    <Nav.Link className="mx-2" href="/settings">Settings</Nav.Link>
                                    <Nav.Link className="mx-2" onClick={handleLogout}>Logout</Nav.Link>
                                </div>
                            )}
                        </Nav>
                    </Navbar.Collapse>
                </Container>
            </Navbar>

            <Container className="mt-3">
                <Routes>
                    <Route path="/" element={<Home isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/invoices/:invoice_id" element={<Invoice/>}/>
                    <Route path="/dashboard" element={<Dashboard isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/about" element={<About/>}/>
                    <Route path="/contact" element={<Contact/>}/>
                    <Route path="/docs" element={<Documentation/>}/>
                    <Route path="/login" element={<Auth onLogin={handleLogin}/>}/>
                    <Route path="/settings" element={<Account />} />
                    <Route path="*" element={<NotFound />} />
                </Routes>
            </Container>
        </>
    );
}

export default App;
