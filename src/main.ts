import App from '@/App.vue';
import '@/assets/index.css';
import router from '@/lib/router';
import pinia from '@/lib/stores';
import '@mdi/font/css/materialdesignicons.css';
import { VueQueryPlugin } from '@tanstack/vue-query';
import 'unfonts.css';
import { createApp } from 'vue';
import 'vue-sonner/style.css';
import { createVuetify } from 'vuetify';
import { zhHans, en } from 'vuetify/locale';
import 'vuetify/styles';

const vuetify = createVuetify({
  icons: {
    defaultSet: 'mdi',
  },
  locale: {
    locale: 'zhHans',
    fallback: 'en',
    messages: { zhHans, en },
  },
  theme: { defaultTheme: 'system' },
});

const app = createApp(App);

app.use(pinia);
app.use(router);
app.use(vuetify);
app.use(VueQueryPlugin, {
  enableDevtoolsV6Plugin: true,
});

app.mount('#root');
