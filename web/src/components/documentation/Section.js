import {Card} from "react-bootstrap";
import React from "react";

const Section = ({section}) => (
    <Card className={`mb-4 ${section.notReady ? "bg-secondary text-white" : "bg-light"}`}>
        <Card.Body>
            <Card.Title id={section.id} className="text-dark h6">
                {section.title}
            </Card.Title>
            {section.notReady ? (
                <Card.Text>
                    <em>This section is under construction. Please check back later.</em>
                </Card.Text>
            ) : (<>{section.content}</>)}
        </Card.Body>
    </Card>
);

export default Section;
