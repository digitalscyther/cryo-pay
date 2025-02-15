import {Nav, Navbar} from "react-bootstrap";
import React from "react";

const Sidebar = ({sections}) => (
    <Navbar sticky="top">
        <Nav className="flex-column">
            {sections.map((section) => (
                <Nav.Link
                    key={section.id}
                    href={`#${section.id}`}
                    className={`text-light ${section.notReady ? "not-ready" : ""}`}
                    title={section.notReady ? "Coming Soon" : ""}
                >
                    {section.title}
                    {section.notReady && <small className="ms-2">(Coming Soon)</small>}
                </Nav.Link>
            ))}
        </Nav>
    </Navbar>
);

export default Sidebar;
