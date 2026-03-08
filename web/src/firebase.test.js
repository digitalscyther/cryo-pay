jest.mock('firebase/app', () => ({
    initializeApp: jest.fn(() => ({})),
}));
jest.mock('firebase/auth', () => ({
    getAuth: jest.fn(() => ({})),
    signInWithEmailAndPassword: jest.fn(),
    signOut: jest.fn(),
    createUserWithEmailAndPassword: jest.fn(),
}));
jest.mock('firebase/analytics', () => ({
    getAnalytics: jest.fn(() => ({})),
}));
jest.mock('./firebaseConfig.json', () => ({}), { virtual: true });

import { getFirebaseErrorMessage } from './firebase';

describe('getFirebaseErrorMessage', () => {
    it('returns message for email-already-in-use', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/email-already-in-use' }))
            .toContain('already in use');
    });

    it('returns message for invalid-email', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/invalid-email' }))
            .toContain('not valid');
    });

    it('returns message for user-not-found', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/user-not-found' }))
            .toContain('No user found');
    });

    it('returns message for wrong-password', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/wrong-password' }))
            .toContain('incorrect');
    });

    it('returns message for weak-password', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/weak-password' }))
            .toContain('too weak');
    });

    it('returns message for too-many-requests', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/too-many-requests' }))
            .toContain('Too many requests');
    });

    it('returns message for invalid-credential', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/invalid-credential' }))
            .toContain('invalid');
    });

    it('returns default message for unknown code', () => {
        expect(getFirebaseErrorMessage({ code: 'auth/unknown-error' }))
            .toContain('unknown error');
    });
});
