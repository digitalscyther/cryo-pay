import React, {useEffect, useState} from 'react';
import {Route, Routes, useLocation, useNavigate, useSearchParams} from 'react-router-dom';
import {Container, Navbar, Nav} from 'react-bootstrap';
import axios from "axios";
import Home from './components/Home';
import About from './components/About';
import Contact from './components/Contact';
import DonationPage from './components/donate/DonationPage';
import Invoice from './components/invoice/Invoice';
import Payment from './components/payment/Payment';
import Auth from './components/Auth';
import Account from './components/settings/Account';
import NotFound from './components/NotFound';
import Dashboard from './components/dashboard/Dashboard';
import {apiUrl, getProjectName} from "./utils";
import Documentation from "./components/documentation/Documentation";

function App() {
    const navigate = useNavigate();
    const location = useLocation();
    const [isLoggedIn, setIsLoggedIn] = useState(false);
    const [searchParams] = useSearchParams();
    const noNavBar = searchParams.get("nnb") === "1";

    useEffect(() => {
        axios
            .get(apiUrl('/user'), {withCredentials: true})
            .then(() => setIsLoggedIn(true))
            .catch(() => setIsLoggedIn(false));
    }, []);

    const isActive = (path) => location.pathname === path;

    const handleLogin = (token) => {
        axios
            .post(apiUrl('/auth/login'), {token}, {withCredentials: true})
            .then(() => {
                setIsLoggedIn(true);
            })
            .catch((err) => {
                console.log("Failed to login", err);
            });
        navigate('/dashboard');
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
        navigate('/dashboard');
    };

    const projectName = getProjectName();
    document.title = projectName;

    return (
        <>
            {!noNavBar && (
                <Navbar bg="dark" variant="dark" expand="lg">
                    <Container>
                        <Navbar.Brand href="/">{projectName}</Navbar.Brand>
                        <Navbar.Toggle aria-controls="basic-navbar-nav"/>
                        <Navbar.Collapse id="basic-navbar-nav">
                            <Nav className="d-flex w-100">
                                <Nav.Link href="/dashboard" active={isActive("/dashboard")}>Dashboard</Nav.Link>
                                <Nav.Link href="/docs" active={isActive("/docs")}>Documentation</Nav.Link>
                                <Nav.Link href="/swagger-ui/" target="_blank" rel="noreferrer">API Docs</Nav.Link>
                                <Nav.Link href="/about" active={isActive("/about")}>About</Nav.Link>
                                <Nav.Link href="/donate" active={isActive("/donate")}>Donate</Nav.Link>
                                <Nav.Link href="/contact" active={isActive("/contact")}>Contact</Nav.Link>
                                <div className="ms-lg-auto d-lg-flex">
                                    {!isLoggedIn ?
                                        <Nav.Link href="/login" active={isActive("/login")}>Login</Nav.Link>
                                        : <>
                                            <Nav.Link className="mx-lg-2" href="/settings"
                                                      active={isActive("/settings")}>Settings</Nav.Link>
                                            <Nav.Link className="mx-lg-2" onClick={handleLogout}>Logout</Nav.Link>
                                        </>}
                                </div>
                            </Nav>
                        </Navbar.Collapse>
                    </Container>
                </Navbar>
            )}

            <Container className="mt-3">
                <Routes>
                    <Route path="/" element={<Home/>}/>
                    <Route path="/invoices/:invoice_id" element={<Invoice/>}/>
                    <Route path="/dashboard" element={<Dashboard isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/about" element={<About/>}/>
                    <Route path="/contact" element={<Contact/>}/>
                    <Route path="/docs" element={<Documentation/>}/>
                    <Route path="/login" element={<Auth onLogin={handleLogin}/>}/>
                    <Route path="/settings" element={<Account/>}/>
                    <Route path="/donate" element={<DonationPage/>}/>
                    <Route path="/payment/:payment_id" element={<Payment/>}/>
                    <Route path="*" element={<NotFound/>}/>
                </Routes>
            </Container>
        </>
    );
}

export default App;
