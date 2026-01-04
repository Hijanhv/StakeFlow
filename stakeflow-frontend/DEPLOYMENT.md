# Frontend Deployment Guide

## Deploy to Vercel (Recommended)

### Option 1: Via Vercel Dashboard (Easiest)

1. **Go to Vercel:** https://vercel.com
2. **Sign in** with GitHub
3. **Click "Add New Project"**
4. **Import** your GitHub repository: `Hijanhv/StakeFlow`
5. **Configure:**
   - Framework Preset: **Next.js**
   - Root Directory: **stakeflow-frontend**
   - Build Command: `npm run build` (auto-detected)
   - Output Directory: `.next` (auto-detected)
6. **Click "Deploy"**
7. **Wait 2-3 minutes** for build and deployment
8. **Get your live URL:** `https://stakeflow-<random>.vercel.app`

### Option 2: Via Vercel CLI

```bash
# Install Vercel CLI
npm i -g vercel

# Navigate to frontend
cd stakeflow-frontend

# Login to Vercel
vercel login

# Deploy
vercel --prod
```

## After Deployment

1. **Copy your live URL** from Vercel dashboard
2. **Update main README.md:**
   - Find: `[Add your Vercel URL here after deployment]`
   - Replace with your actual URL
3. **Commit and push:**
   ```bash
   git add README.md
   git commit -m "Add live demo URL"
   git push
   ```

## Custom Domain (Optional)

1. Go to your Vercel project settings
2. Click "Domains"
3. Add your custom domain (e.g., `stakeflow.app`)
4. Follow DNS configuration instructions

## Environment Variables (If Needed Later)

When you integrate real Casper wallet:
1. Go to Vercel project → Settings → Environment Variables
2. Add:
   - `NEXT_PUBLIC_CASPER_NETWORK=casper-test`
   - `NEXT_PUBLIC_CONTRACT_HASH=<your-contract-hash>`

## Troubleshooting

### Build Fails
- Check `npm run build` works locally first
- Ensure all dependencies are in `package.json`
- Check Node version (use 18.x or 20.x)

### Image Not Loading
- Ensure `logo.png` is in `/public` folder
- Check image path is `/logo.png` not `./logo.png`

### Deployment Taking Too Long
- Normal for first deployment (2-5 minutes)
- Subsequent deploys are faster (~30 seconds)

## Features to Add After Deployment

1. **Connect to Real Contract:**
   - Update contract hash in code
   - Add Casper wallet integration (CSPR.click)

2. **Add Analytics:**
   - Vercel Analytics (free)
   - Google Analytics

3. **SEO Improvements:**
   - Add Open Graph images
   - Update meta descriptions

## Support

- **Vercel Docs:** https://vercel.com/docs
- **Next.js Docs:** https://nextjs.org/docs
- **GitHub Issues:** https://github.com/Hijanhv/StakeFlow/issues
