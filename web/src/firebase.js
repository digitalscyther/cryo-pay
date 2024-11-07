import { initializeApp } from "firebase/app";
import {
    getAuth,
    signInWithEmailAndPassword,
    signOut,
    createUserWithEmailAndPassword,
} from 'firebase/auth';
import { getAnalytics } from "firebase/analytics";
import firebaseConfig from "./firebaseConfig.json";

const app = initializeApp(firebaseConfig);
const auth = getAuth(app);
const analytics = getAnalytics(app);

function getFirebaseErrorMessage(error) {
    switch (error.code) {
        case 'auth/email-already-in-use':
            return 'This email is already in use. Please use a different email.';
        case 'auth/invalid-email':
            return 'The provided email is not valid. Please check and try again.';
        case 'auth/user-not-found':
            return 'No user found with this email. Please check your credentials.';
        case 'auth/wrong-password':
            return 'The password you entered is incorrect. Please try again.';
        case 'auth/operation-not-allowed':
            return 'This operation is not allowed. Please contact support.';
        case 'auth/weak-password':
            return 'The password is too weak. Please choose a stronger password.';
        case 'auth/too-many-requests':
            return 'Too many requests have been made. Please try again later.';
        case 'auth/invalid-credential':
            return 'The credentials provided are invalid. Please check your login details and try again.';
        default:
            return 'An unknown error occurred. Please try again later.';
    }
}

export {
    analytics,
    auth,
    signInWithEmailAndPassword,
    signOut,
    createUserWithEmailAndPassword,
    getFirebaseErrorMessage,
};