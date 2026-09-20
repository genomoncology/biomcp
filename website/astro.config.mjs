import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://biomcp.org',
  integrations: [
    starlight({
      title: 'BioMCP',
      sidebar: [{ label: 'BioData models', items: [{ label: 'ClinicalTrial', slug: 'biodata/models/clinical-trial' }, { label: 'ScientificPublication', slug: 'biodata/models/scientific-publication' }] }],
    }),
  ],
});
