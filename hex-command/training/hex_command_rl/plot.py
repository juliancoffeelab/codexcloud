"""Plot a saved run without rerunning training or match evaluation."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('report',type=Path)
    parser.add_argument('--out',type=Path)
    args=parser.parse_args()
    report=json.loads(args.report.read_text()); history=report['history']
    output=args.out or args.report.with_name('learning-curves.png')
    times=[0]+[r['seconds'] for r in history]
    validation=[report['initial_validation']]+[r['validation'] for r in history]
    training=[report['initial_train_probe']]+[r['train_probe'] for r in history]
    selected=next(r for r in history if r['epoch']==report['best_epoch'])
    plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
    fig,axes=plt.subplots(2,2,figsize=(13,8))
    for ax in axes.flat:ax.grid(axis='y',alpha=.2);ax.set_axisbelow(True)
    ax=axes[0,0]
    ax.plot(times,[r['cross_entropy'] for r in training],label='Fixed training sample (1,024 max)')
    ax.plot(times,[r['cross_entropy'] for r in validation],label='Held-out validation')
    ax.axvline(selected['seconds'],color='purple',ls=':',label='Selected checkpoint')
    ax.set(title='Imitation loss',xlabel='Elapsed seconds',ylabel='Cross-entropy');ax.legend(fontsize=8)
    ax=axes[0,1]
    for name in ('move','attack','end_turn','advance'):
        ax.plot(times,[100*r['by_action'][name][0]/max(1,r['by_action'][name][1]) for r in validation],label=name,alpha=.8)
    ax.plot(times,[100*r['accuracy'] for r in validation],label='Overall',color='black',lw=2)
    ax.set(title='Held-out agreement by teacher action',xlabel='Elapsed seconds',ylabel='Accuracy (%)',ylim=(0,102));ax.legend(fontsize=8)
    ax=axes[1,0]
    for name in ('move','attack','end_turn','advance'):
        ax.plot(times,[100*r['predicted_actions'][name]/max(1,r['decisions']) for r in validation],label=name)
    ax.set(title='Predicted action frequencies on held-out positions',xlabel='Elapsed seconds',ylabel='Predictions (%)',ylim=(0,100));ax.legend(fontsize=8)
    ax=axes[1,1];periodic=report['periodic_history']
    if periodic:
        x=[r['seconds'] for r in periodic]
        for key in ('wins','losses','caps'):
            ax.plot(x,[100*r['learned'][key]/r['learned']['games'] for r in periodic],marker='o',label=key)
        b=periodic[0]['baseline'];ax.axhline(100*b['wins']/b['games'],ls='--',color='grey',label='Baseline wins')
    ax.set(title='Periodic validation matches (30-round cap)',xlabel='Elapsed seconds',ylabel='Matches (%)',ylim=(0,100));ax.legend(fontsize=8)
    final=report['learned_results'];wins=sum(r['success'] for r in final);caps=sum(r['truncated'] for r in final)
    limit=report['evaluation_configs'][0]['maxRounds']
    fig.suptitle(f"Native deathmatch imitation · schema {report['schema']} · {len(report['feature_names'])} inputs",fontsize=16)
    fig.text(.04,.015,f"Final evaluation ({limit}-round cap, {len(final)} matches): {wins} wins / {len(final)-wins-caps} losses / {caps} caps. Periodic matches use separate seeds.",fontsize=9)
    fig.tight_layout(rect=(0,.04,1,.95));output.parent.mkdir(parents=True,exist_ok=True)
    fig.savefig(output,dpi=170);print(output)


if __name__=='__main__':main()
