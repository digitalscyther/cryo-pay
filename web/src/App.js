import React, {useEffect, useState} from 'react';
import {Route, Routes, useLocation, useNavigate} from 'react-router-dom';
import {Container, Navbar, Nav} from 'react-bootstrap';
import axios from "axios";
import {jwtDecode} from "jwt-decode";
import Cookies from "js-cookie";
import Home from './components/Home';
import About from './components/About';
import Contact from './components/Contact';
import Invoice from './components/invoice/Invoice';
import Auth from './components/Auth';
import Account from './components/settings/Account';
import NotFound from './components/NotFound';
import Dashboard from './components/dashboard/Dashboard';
import {apiUrl, getProjectName} from "./utils";
import Documentation from "./components/Documentation";

function App() {
    const navigate = useNavigate();
    const location = useLocation();
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
            <Navbar bg="dark" variant="dark" expand="lg">
                <Container>
                    <Navbar.Brand href="/">{projectName}</Navbar.Brand>
                    <Navbar.Toggle aria-controls="basic-navbar-nav"/>
                    <Navbar.Collapse id="basic-navbar-nav">
                        <Nav className="d-flex w-100">
                            <Nav.Link href="/dashboard" active={isActive("/dashboard")}>Dashboard</Nav.Link>
                            <Nav.Link href="/docs" active={isActive("/docs")}>Documentation</Nav.Link>
                            <Nav.Link href="/about" active={isActive("/about")}>About</Nav.Link>
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

            <Container className="mt-3">
                <Routes>
                    <Route path="/" element={<Home isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/invoices/:invoice_id" element={<Invoice/>}/>
                    <Route path="/dashboard" element={<Dashboard isLoggedIn={isLoggedIn}/>}/>
                    <Route path="/about" element={<About/>}/>
                    <Route path="/contact" element={<Contact/>}/>
                    <Route path="/docs" element={<Documentation/>}/>
                    <Route path="/login" element={<Auth onLogin={handleLogin}/>}/>
                    <Route path="/settings" element={<Account/>}/>
                    <Route path="*" element={<NotFound/>}/>
                </Routes>
            </Container>
        </>
    );
}

export default App;
