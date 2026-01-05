describe('Static Import Resolution', () => {
  it('should resolve local static imports and avoid errors', () => {
    cy.visit('/', {
      onBeforeLoad(win) {
        cy.stub(win.console, 'error').as('consoleError');
        cy.stub(win.console, 'log').as('consoleLog');
      }
    });

    // Wait for the app to report status
    cy.get('#e2e-status', { timeout: 20000 })
      .should('exist')
      .should('contain.text', 'static-imports-ok');

    // Assert status endpoint reports success
    cy.request('/e2e/status')
      .its('body')
      .then((body) => {
        expect(body).to.have.property('success', true);
        expect(body).to.have.nested.property('details.missing');
        expect(body.details.missing).to.have.length(0);
        expect(body).to.not.have.nested.property('details.lastErr');
        // Cache should contain our expected local specs
        const keys = body.details.cacheKeys || [];
        // We check for the presence of these files in the cache
        const hasListItem = keys.some(k => k.includes('list-item.jsx'));
        const hasSampleData = keys.some(k => k.includes('sample-data.js'));
        const hasNsHelper = keys.some(k => k.includes('ns-helper.js'));

        expect(hasListItem).to.equal(true, 'Cache should include list-item.jsx');
        expect(hasSampleData).to.equal(true, 'Cache should include sample-data.js');
        expect(hasNsHelper).to.equal(true, 'Cache should include ns-helper.js');
      });

    // Ensure no static import error was logged to console
    cy.get('@consoleError').then(stub => {
      const calls = stub.getCalls();
      const hasStaticErr = calls.some(call => call.args.join(' ').includes('Failed to load static import'));
      expect(hasStaticErr).to.equal(false);
    });
  });
});
